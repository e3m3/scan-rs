// Copyright 2025, Giordano Salvador
// SPDX-License-Identifier: BSD-3-Clause

#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(unused_mut)]
#![allow(unused_variables)]

use std::num::NonZero;

use bytemuck::bytes_of;
use bytemuck::cast_slice;
use bytemuck::Pod;
use env_logger::init as init_logger;
use pollster::block_on;
use support::copy_casted;
use support::DoubleBufferMode;
use support::IAdd;
use support::ICast;
use support::IDisplay;
use support::IScan;
use wgpu::include_wgsl;
use wgpu::util::BufferInitDescriptor;
use wgpu::util::DeviceExt;
use wgpu::BindGroupDescriptor;
use wgpu::BindGroupEntry;
use wgpu::BindGroupLayoutDescriptor;
use wgpu::BindGroupLayoutEntry;
use wgpu::BindingType;
use wgpu::BufferBindingType;
use wgpu::BufferDescriptor;
use wgpu::BufferUsages;
use wgpu::CommandEncoderDescriptor;
use wgpu::ComputePassDescriptor;
use wgpu::ComputePipelineDescriptor;
use wgpu::DeviceDescriptor;
use wgpu::DownlevelFlags;
use wgpu::Instance;
use wgpu::InstanceDescriptor;
use wgpu::Limits;
use wgpu::Maintain;
use wgpu::MapMode;
use wgpu::MemoryHints;
use wgpu::PipelineCompilationOptions;
use wgpu::PipelineLayoutDescriptor;
use wgpu::RequestAdapterOptions;
use wgpu::ShaderStages;

#[derive(Clone, Copy)]
pub struct Scan {
    verbose: bool,
}

impl Scan {
    /// Implement the parallel GPU exclusive scan algorithm
    pub fn process<T, const N: usize>(
        &self,
        def: T,
        v_in: &[T],
        v_out: &mut [T],
    ) -> Result<(), String>
    where
        T: Copy + Eq + IAdd + ICast<i32> + IDisplay + Ord + Pod + Send,
        i32: ICast<T>,
    {
        const WORKGROUP_SIZE: usize = 64;
        let n_in = v_in.len();
        let n_out = v_out.len();
        let num_chunks = usize::div_ceil(n_out, WORKGROUP_SIZE);
        let d_end = (n_out as f32).log2().ceil() as u32;
        let mode = DoubleBufferMode::default();

        if self.verbose {
            eprintln!("Starting par_gpu_naive_dbl");
            eprintln!("v_in: {:?}", v_in);
            eprintln!("v_out: {:?}", v_out);
        }

        Self::check_args(n_in, n_out)?;
        if v_in.iter().max().cloned().unwrap_or(def) > i32::MAX.cast() {
            return Err(format!(
                "Values in input larger than max supported GPU value ({}:i32)",
                i32::MAX
            ));
        }

        let mut v_in_gpu: Vec<i32> = vec![def.cast(); n_in];
        copy_casted::<T, i32>(&v_in[..(n_in - 1)], &mut v_in_gpu[1..n_in])?;

        if self.verbose {
            eprintln!("v_in_gpu: {:?}", v_in_gpu);
        }

        init_logger();

        let instance = Instance::new(&InstanceDescriptor::default());
        let adapter = block_on(instance.request_adapter(&RequestAdapterOptions::default()))
            .ok_or("Failed to request adapter".to_string())?;

        if self.verbose {
            eprintln!("Found adapter: {:#?}", adapter.get_info());
        }

        if !adapter
            .get_downlevel_capabilities()
            .flags
            .contains(DownlevelFlags::COMPUTE_SHADERS)
        {
            return Err("Adapter does not support compute shaders".to_string());
        }

        let (device, queue) = block_on(adapter.request_device(
            &DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: Limits::downlevel_defaults(),
                memory_hints: MemoryHints::MemoryUsage,
            },
            None,
        ))
        .or(Err("Failed to request adapter".to_string()))?;

        let module = device.create_shader_module(include_wgsl!("shader.wgsl"));
        let input_n_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytes_of(&n_out),
            usage: BufferUsages::UNIFORM,
        });
        let input_N_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytes_of(&N),
            usage: BufferUsages::UNIFORM,
        });
        let input_d_end_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytes_of(&d_end),
            usage: BufferUsages::UNIFORM,
        });
        let input_mode_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytes_of(&mode),
            usage: BufferUsages::STORAGE,
        });
        let input_data_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: cast_slice(&v_in_gpu),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
        });
        let output_data_buffer = device.create_buffer(&BufferDescriptor {
            label: None,
            size: input_data_buffer.size(),
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let download_buffer_a = device.create_buffer(&BufferDescriptor {
            label: None,
            size: input_data_buffer.size(),
            usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let download_buffer_b = device.create_buffer(&BufferDescriptor {
            label: None,
            size: input_data_buffer.size(),
            usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let input_entry_n = BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                min_binding_size: Some(NonZero::new(size_of::<u32>() as u64).unwrap()),
                has_dynamic_offset: false,
            },
            count: None,
        };
        let input_entry_N = BindGroupLayoutEntry {
            binding: 1,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                min_binding_size: Some(NonZero::new(size_of::<u32>() as u64).unwrap()),
                has_dynamic_offset: false,
            },
            count: None,
        };
        let input_entry_d_end = BindGroupLayoutEntry {
            binding: 2,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                min_binding_size: Some(NonZero::new(size_of::<u32>() as u64).unwrap()),
                has_dynamic_offset: false,
            },
            count: None,
        };
        let input_entry_mode = BindGroupLayoutEntry {
            binding: 3,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: false },
                min_binding_size: Some(NonZero::new(size_of::<u32>() as u64).unwrap()),
                has_dynamic_offset: false,
            },
            count: None,
        };
        let input_entry_data = BindGroupLayoutEntry {
            binding: 4,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: false },
                min_binding_size: Some(NonZero::new(size_of::<i32>() as u64).unwrap()),
                has_dynamic_offset: false,
            },
            count: None,
        };
        let output_entry_data = BindGroupLayoutEntry {
            binding: 5,
            visibility: ShaderStages::COMPUTE,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: false },
                min_binding_size: Some(NonZero::new(size_of::<i32>() as u64).unwrap()),
                has_dynamic_offset: false,
            },
            count: None,
        };
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                input_entry_n,
                input_entry_N,
                input_entry_d_end,
                input_entry_mode,
                input_entry_data,
                output_entry_data,
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            module: &module,
            entry_point: Some("scan"),
            compilation_options: PipelineCompilationOptions::default(),
            cache: None,
        });

        let input_bind_group_n = BindGroupEntry {
            binding: 0,
            resource: input_n_buffer.as_entire_binding(),
        };
        let input_bind_group_N = BindGroupEntry {
            binding: 1,
            resource: input_N_buffer.as_entire_binding(),
        };
        let input_bind_group_d_end = BindGroupEntry {
            binding: 2,
            resource: input_d_end_buffer.as_entire_binding(),
        };
        let input_bind_group_mode = BindGroupEntry {
            binding: 3,
            resource: input_mode_buffer.as_entire_binding(),
        };
        let input_bind_group_data = BindGroupEntry {
            binding: 4,
            resource: input_data_buffer.as_entire_binding(),
        };
        let output_bind_group_data = BindGroupEntry {
            binding: 5,
            resource: output_data_buffer.as_entire_binding(),
        };
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                input_bind_group_n,
                input_bind_group_N,
                input_bind_group_d_end,
                input_bind_group_mode,
                input_bind_group_data,
                output_bind_group_data,
            ],
        });

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor { label: None });

        encoder.copy_buffer_to_buffer(
            &input_data_buffer,
            0,
            &output_data_buffer,
            0,
            output_data_buffer.size(),
        );

        let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        });
        compute_pass.set_pipeline(&pipeline);
        compute_pass.set_bind_group(0, &bind_group, &[]);
        compute_pass.dispatch_workgroups(num_chunks as u32, 1, 1);
        drop(compute_pass);

        encoder.copy_buffer_to_buffer(
            &output_data_buffer,
            0,
            &download_buffer_a,
            0,
            output_data_buffer.size(),
        );
        encoder.copy_buffer_to_buffer(
            &input_data_buffer,
            0,
            &download_buffer_b,
            0,
            input_data_buffer.size(),
        );

        let command_buffer = encoder.finish();
        queue.submit([command_buffer]);

        let buffer_slice_a = download_buffer_a.slice(..);
        let buffer_slice_b = download_buffer_b.slice(..);
        buffer_slice_a.map_async(MapMode::Read, |_| {});
        buffer_slice_b.map_async(MapMode::Read, |_| {});

        let _ = device.poll(Maintain::Wait);
        let data_a = buffer_slice_a.get_mapped_range();
        let data_b = buffer_slice_b.get_mapped_range();
        if d_end % 2 == 1 {
            copy_casted::<i32, T>(cast_slice(&data_a), v_out)?;
        } else {
            copy_casted::<i32, T>(cast_slice(&data_b), v_out)?;
        }

        if self.verbose {
            eprintln!("data_a: {:?}", &data_a[..]);
            eprintln!("data_b: {:?}", &data_b[..]);
        }

        Ok(())
    }
}

impl IScan for Scan {
    fn new(verbose: bool) -> Self {
        Self { verbose }
    }
}
