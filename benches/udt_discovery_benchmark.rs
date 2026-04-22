// udt_discovery_benchmark.rs - Performance benchmarks for UDT discovery
// =========================================================================
//
// This file contains performance benchmarks for the UDT discovery and
// hierarchical tag access features of the Rust EtherNet/IP library.

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use rust_ethernet_ip::{TagMetadata, TagPermissions, TagScope, UdtDefinition, UdtMember};
use std::collections::HashMap;
use std::time::Instant;

/// Mock UDT definitions for benchmarking
fn create_mock_udt_definitions(count: usize) -> HashMap<String, UdtDefinition> {
    let mut definitions = HashMap::new();

    for i in 0..count {
        let udt_name = format!("UDT_{:04}", i);
        let members = vec![
            UdtMember {
                name: "Running".to_string(),
                data_type: 0x00C1,
                offset: 0,
                size: 1,
            },
            UdtMember {
                name: "Speed".to_string(),
                data_type: 0x00CA,
                offset: 4,
                size: 4,
            },
            UdtMember {
                name: "Position".to_string(),
                data_type: 0x00C4,
                offset: 8,
                size: 4,
            },
            UdtMember {
                name: "Temperature".to_string(),
                data_type: 0x00CA,
                offset: 12,
                size: 4,
            },
            UdtMember {
                name: "Status".to_string(),
                data_type: 0x00C4,
                offset: 16,
                size: 4,
            },
        ];

        definitions.insert(
            udt_name.clone(),
            UdtDefinition {
                name: udt_name,
                members,
            },
        );
    }

    definitions
}

/// Benchmark UDT member discovery
fn benchmark_udt_member_discovery(c: &mut Criterion) {
    let mut group = c.benchmark_group("udt_member_discovery");

    for udt_count in [10, 100, 1000, 10000].iter() {
        let udt_definitions = create_mock_udt_definitions(*udt_count);

        group.bench_with_input(
            BenchmarkId::new("discover_members", udt_count),
            udt_count,
            |b, _| {
                b.iter(|| {
                    let mut all_tags = Vec::new();
                    let mut tag_names = std::collections::HashSet::new();

                    for (udt_name, udt_def) in &udt_definitions {
                        for member in &udt_def.members {
                            let full_name = format!("{}.{}", udt_name, member.name);

                            if !tag_names.contains(&full_name) {
                                let metadata = TagMetadata {
                                    data_type: member.data_type,
                                    scope: TagScope::Controller,
                                    permissions: TagPermissions {
                                        readable: true,
                                        writable: true,
                                    },
                                    is_array: false,
                                    dimensions: Vec::new(),
                                    last_access: Instant::now(),
                                    size: member.size,
                                    array_info: None,
                                    last_updated: Instant::now(),
                                };

                                all_tags.push((full_name.clone(), metadata));
                                tag_names.insert(full_name);
                            }
                        }
                    }

                    black_box(all_tags)
                })
            },
        );
    }

    group.finish();
}

/// Benchmark tag name validation
fn benchmark_tag_name_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("tag_name_validation");

    let test_names = vec![
        "ValidTag",
        "Valid_Tag",
        "Valid.Tag",
        "Valid123",
        "Invalid123",
        "Invalid__Tag",
        "Invalid..Tag",
        "Invalid-Tag",
        "Invalid Tag",
        "Invalid@Tag",
    ];

    group.bench_function("validate_names", |b| {
        b.iter(|| {
            for name in &test_names {
                // Simulate tag name validation logic
                let is_valid = !name.is_empty()
                    && !name.trim().is_empty()
                    && !name.starts_with(char::is_numeric)
                    && !name.contains("__")
                    && !name.contains("..")
                    && !name.contains('-')
                    && !name.contains(' ')
                    && !name.contains('@');

                black_box(is_valid);
            }
        })
    });

    group.finish();
}

/// Benchmark UDT definition caching
fn benchmark_udt_caching(c: &mut Criterion) {
    let mut group = c.benchmark_group("udt_caching");

    let udt_definitions = create_mock_udt_definitions(1000);

    group.bench_function("cache_operations", |b| {
        b.iter(|| {
            let mut cache = HashMap::new();

            // Insert UDT definitions
            for (name, definition) in &udt_definitions {
                cache.insert(name.clone(), definition.clone());
            }

            // Retrieve UDT definitions
            for name in udt_definitions.keys() {
                let _definition = cache.get(name);
                black_box(_definition);
            }

            // Clear cache
            cache.clear();

            black_box(cache)
        })
    });

    group.finish();
}

/// Benchmark hierarchical tag discovery
fn benchmark_hierarchical_discovery(c: &mut Criterion) {
    let mut group = c.benchmark_group("hierarchical_discovery");

    for depth in [1, 2, 3, 4].iter() {
        group.bench_with_input(
            BenchmarkId::new("discovery_depth", depth),
            depth,
            |b, depth| {
                b.iter(|| {
                    let mut all_tags = Vec::new();
                    let mut tag_names = std::collections::HashSet::new();

                    // Simulate hierarchical discovery
                    for level in 0..*depth {
                        let base_name = if level == 0 {
                            "BaseUDT".to_string()
                        } else {
                            format!("BaseUDT.Level{}", level)
                        };

                        for member_num in 0..5 {
                            let member_name = format!("Member{}", member_num);
                            let full_name = format!("{}.{}", base_name, member_name);

                            if !tag_names.contains(&full_name) {
                                let metadata = TagMetadata {
                                    data_type: 0x00C4,
                                    scope: TagScope::Controller,
                                    permissions: TagPermissions {
                                        readable: true,
                                        writable: true,
                                    },
                                    is_array: false,
                                    dimensions: Vec::new(),
                                    last_access: Instant::now(),
                                    size: 4,
                                    array_info: None,
                                    last_updated: Instant::now(),
                                };

                                all_tags.push((full_name.clone(), metadata));
                                tag_names.insert(full_name);
                            }
                        }
                    }

                    black_box(all_tags)
                })
            },
        );
    }

    group.finish();
}

/// Benchmark UDT response parsing
fn benchmark_udt_response_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("udt_response_parsing");

    // Create mock response data
    let response_data: Vec<u8> = (0..1000)
        .flat_map(|i| {
            let data_type = match i % 4 {
                0 => 0x00C1, // BOOL
                1 => 0x00C4, // DINT
                2 => 0x00CA, // REAL
                _ => 0x00CE, // STRING
            };

            vec![
                (data_type & 0xFF) as u8,
                ((data_type >> 8) & 0xFF) as u8,
                0x00,
                0x00,
            ]
        })
        .collect();

    group.bench_function("parse_response", |b| {
        b.iter(|| {
            let mut offset = 0;
            let mut found_types = Vec::new();

            while offset < response_data.len().saturating_sub(4) {
                let data_type =
                    u16::from_le_bytes([response_data[offset], response_data[offset + 1]]);
                found_types.push(data_type);
                offset += 4;
            }

            black_box(found_types)
        })
    });

    group.finish();
}

/// Benchmark large-scale tag discovery
fn benchmark_large_scale_discovery(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_scale_discovery");

    for tag_count in [1000, 10000, 100000].iter() {
        group.bench_with_input(
            BenchmarkId::new("discover_tags", tag_count),
            tag_count,
            |b, tag_count| {
                b.iter(|| {
                    let mut all_tags = Vec::new();
                    let mut tag_names = std::collections::HashSet::new();

                    // Simulate discovering a large number of tags
                    for i in 0..*tag_count {
                        let tag_name = format!("Tag_{:06}", i);

                        if !tag_names.contains(&tag_name) {
                            let metadata = TagMetadata {
                                data_type: 0x00C4,
                                scope: TagScope::Controller,
                                permissions: TagPermissions {
                                    readable: true,
                                    writable: true,
                                },
                                is_array: false,
                                dimensions: Vec::new(),
                                last_access: Instant::now(),
                                size: 4,
                                array_info: None,
                                last_updated: Instant::now(),
                            };

                            all_tags.push((tag_name.clone(), metadata));
                            tag_names.insert(tag_name);
                        }
                    }

                    black_box(all_tags)
                })
            },
        );
    }

    group.finish();
}

/// Benchmark memory usage for UDT discovery
fn benchmark_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");

    group.bench_function("udt_storage", |b| {
        b.iter(|| {
            let mut definitions = HashMap::new();

            // Create 1000 UDT definitions with 5 members each
            for i in 0..1000 {
                let udt_name = format!("UDT_{:04}", i);
                let members = vec![
                    UdtMember {
                        name: "Member1".to_string(),
                        data_type: 0x00C1,
                        offset: 0,
                        size: 1,
                    },
                    UdtMember {
                        name: "Member2".to_string(),
                        data_type: 0x00C4,
                        offset: 4,
                        size: 4,
                    },
                    UdtMember {
                        name: "Member3".to_string(),
                        data_type: 0x00CA,
                        offset: 8,
                        size: 4,
                    },
                    UdtMember {
                        name: "Member4".to_string(),
                        data_type: 0x00C4,
                        offset: 12,
                        size: 4,
                    },
                    UdtMember {
                        name: "Member5".to_string(),
                        data_type: 0x00CA,
                        offset: 16,
                        size: 4,
                    },
                ];

                definitions.insert(
                    udt_name,
                    UdtDefinition {
                        name: format!("UDT_{:04}", i),
                        members,
                    },
                );
            }

            black_box(definitions)
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_udt_member_discovery,
    benchmark_tag_name_validation,
    benchmark_udt_caching,
    benchmark_hierarchical_discovery,
    benchmark_udt_response_parsing,
    benchmark_large_scale_discovery,
    benchmark_memory_usage
);

criterion_main!(benches);
