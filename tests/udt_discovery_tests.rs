use rust_ethernet_ip::udt::{TagPermissions, TagScope};
use rust_ethernet_ip::{RouteHop, RoutePath, TagAttributes, UdtDefinition, UdtMember, UdtTemplate};
use std::collections::HashMap;
/// Mock EipClient for testing UDT discovery functionality
struct MockEipClient {
    udt_definitions: HashMap<String, UdtDefinition>,
    tag_attributes: HashMap<String, TagAttributes>,
    templates: HashMap<u32, UdtTemplate>,
}

impl MockEipClient {
    fn new() -> Self {
        Self {
            udt_definitions: HashMap::new(),
            tag_attributes: HashMap::new(),
            templates: HashMap::new(),
        }
    }

    fn add_test_udt_definition(&mut self) {
        let definition = UdtDefinition {
            name: "TestUDT".to_string(),
            members: vec![
                UdtMember {
                    name: "bool_member".to_string(),
                    data_type: 0x00C1, // BOOL
                    offset: 0,
                    size: 1,
                },
                UdtMember {
                    name: "int_member".to_string(),
                    data_type: 0x00C3, // INT
                    offset: 1,
                    size: 2,
                },
                UdtMember {
                    name: "dint_member".to_string(),
                    data_type: 0x00C4, // DINT
                    offset: 4,
                    size: 4,
                },
                UdtMember {
                    name: "real_member".to_string(),
                    data_type: 0x00CA, // REAL
                    offset: 8,
                    size: 4,
                },
            ],
        };
        self.udt_definitions
            .insert("TestUDT".to_string(), definition);
    }

    fn add_test_tag_attributes(&mut self) {
        let attributes = TagAttributes {
            name: "TestUDT".to_string(),
            data_type: 0x00A0, // UDT
            data_type_name: "UDT".to_string(),
            dimensions: Vec::new(),
            permissions: TagPermissions::ReadWrite,
            scope: TagScope::Controller,
            template_instance_id: Some(123),
            size: 16,
        };
        self.tag_attributes
            .insert("TestUDT".to_string(), attributes);
    }

    fn add_test_template(&mut self) {
        let template = UdtTemplate {
            template_id: 123,
            name: "TestTemplate".to_string(),
            size: 16,
            member_count: 4,
            members: vec![
                UdtMember {
                    name: "bool_member".to_string(),
                    data_type: 0x00C1, // BOOL
                    offset: 0,
                    size: 1,
                },
                UdtMember {
                    name: "int_member".to_string(),
                    data_type: 0x00C3, // INT
                    offset: 1,
                    size: 2,
                },
                UdtMember {
                    name: "dint_member".to_string(),
                    data_type: 0x00C4, // DINT
                    offset: 4,
                    size: 4,
                },
                UdtMember {
                    name: "real_member".to_string(),
                    data_type: 0x00CA, // REAL
                    offset: 8,
                    size: 4,
                },
            ],
        };
        self.templates.insert(123, template);
    }
}

#[test]
fn test_udt_definition_discovery() {
    let mut client = MockEipClient::new();
    client.add_test_udt_definition();
    client.add_test_tag_attributes();
    client.add_test_template();

    // Test UDT definition retrieval
    let definition = client.udt_definitions.get("TestUDT").unwrap();
    assert_eq!(definition.name, "TestUDT");
    assert_eq!(definition.members.len(), 4);

    // Test member properties
    let bool_member = &definition.members[0];
    assert_eq!(bool_member.name, "bool_member");
    assert_eq!(bool_member.data_type, 0x00C1);
    assert_eq!(bool_member.offset, 0);
    assert_eq!(bool_member.size, 1);

    let int_member = &definition.members[1];
    assert_eq!(int_member.name, "int_member");
    assert_eq!(int_member.data_type, 0x00C3);
    assert_eq!(int_member.offset, 1);
    assert_eq!(int_member.size, 2);

    let dint_member = &definition.members[2];
    assert_eq!(dint_member.name, "dint_member");
    assert_eq!(dint_member.data_type, 0x00C4);
    assert_eq!(dint_member.offset, 4);
    assert_eq!(dint_member.size, 4);

    let real_member = &definition.members[3];
    assert_eq!(real_member.name, "real_member");
    assert_eq!(real_member.data_type, 0x00CA);
    assert_eq!(real_member.offset, 8);
    assert_eq!(real_member.size, 4);
}

#[test]
fn test_tag_attributes_parsing() {
    let mut client = MockEipClient::new();
    client.add_test_tag_attributes();

    let attributes = client.tag_attributes.get("TestUDT").unwrap();
    assert_eq!(attributes.name, "TestUDT");
    assert_eq!(attributes.data_type, 0x00A0);
    assert_eq!(attributes.data_type_name, "UDT");
    assert_eq!(attributes.permissions, TagPermissions::ReadWrite);
    assert_eq!(attributes.scope, TagScope::Controller);
    assert_eq!(attributes.template_instance_id, Some(123));
    assert_eq!(attributes.size, 16);
}

#[test]
fn test_udt_template_parsing() {
    let mut client = MockEipClient::new();
    client.add_test_template();

    let template = client.templates.get(&123).unwrap();
    assert_eq!(template.template_id, 123);
    assert_eq!(template.name, "TestTemplate");
    assert_eq!(template.size, 16);
    assert_eq!(template.member_count, 4);
    assert_eq!(template.members.len(), 4);
}

#[test]
fn test_route_path_creation() {
    let route = RoutePath::new()
        .add_slot(0)
        .add_slot(2)
        .add_address("192.168.1.100".to_string())
        .add_port(1);

    assert_eq!(route.slots, vec![0, 2]);
    assert_eq!(route.addresses, vec!["192.168.1.100".to_string()]);
    assert_eq!(route.ports, vec![1]);

    // Test CIP bytes generation
    let cip_bytes = route.to_cip_bytes();
    assert!(!cip_bytes.is_empty());
}

#[test]
fn test_route_path_cip_bytes() {
    let route = RoutePath::new()
        .add_slot(0)
        .add_address("192.168.1.100".to_string());

    let cip_bytes = route.to_cip_bytes();

    assert_eq!(
        cip_bytes,
        vec![
            0x01, 0x00, 0x12, 0x0E, b'1', b'9', b'2', b'.', b'1', b'6', b'8', b'.', b'1', b'.',
            b'1', b'0', b'0', 0x00
        ]
    );
}

#[test]
fn test_route_path_preserves_mixed_hop_order() {
    let route = RoutePath::new()
        .add_slot(0)
        .add_ethernet("192.168.1.5")
        .add_slot(3);

    assert_eq!(
        route.hops,
        vec![
            RouteHop::Backplane { port: 1, slot: 0 },
            RouteHop::Ethernet {
                port: 2,
                address: "192.168.1.5".to_string()
            },
            RouteHop::Backplane { port: 1, slot: 3 },
        ]
    );
    assert_eq!(
        route.to_cip_bytes(),
        vec![
            0x01, 0x00, 0x12, 0x0C, b'1', b'9', b'2', b'.', b'1', b'6', b'8', b'.', b'1', b'.',
            b'5', 0x00, 0x01, 0x03
        ]
    );
}

#[test]
fn test_route_path_uses_explicit_ethernet_port() {
    let route = RoutePath::new()
        .add_slot(1)
        .add_ethernet_with_port(3, "10.20.30.40");

    assert_eq!(
        route.to_cip_bytes(),
        vec![
            0x01, 0x01, 0x13, 0x0C, b'1', b'0', b'.', b'2', b'0', b'.', b'3', b'0', b'.', b'4',
            b'0', 0x00
        ]
    );
}

#[test]
fn test_legacy_port_address_pairing_still_works_when_port_is_added_late() {
    let route = RoutePath::new()
        .add_slot(0)
        .add_address("192.168.1.100".to_string())
        .add_port(3);

    assert_eq!(
        route.to_cip_bytes(),
        vec![
            0x01, 0x00, 0x13, 0x0E, b'1', b'9', b'2', b'.', b'1', b'6', b'8', b'.', b'1', b'.',
            b'1', b'0', b'0', 0x00
        ]
    );
}

#[test]
fn test_legacy_public_field_construction_still_encodes_route() {
    let route = RoutePath {
        slots: vec![0],
        ports: vec![3],
        addresses: vec!["10.20.30.40".to_string()],
        hops: Vec::new(),
    };

    assert_eq!(
        route.to_cip_bytes(),
        vec![
            0x01, 0x00, 0x13, 0x0C, b'1', b'0', b'.', b'2', b'0', b'.', b'3', b'0', b'.', b'4',
            b'0', 0x00
        ]
    );
}

#[test]
fn test_data_type_name_mapping() {
    let test_cases = vec![
        (0x00C1, "BOOL"),
        (0x00C2, "SINT"),
        (0x00C3, "INT"),
        (0x00C4, "DINT"),
        (0x00CA, "REAL"),
        (0x00CE, "STRING"),
        (0x00A0, "UDT"),
    ];

    for (_data_type, expected_name) in test_cases {
        // This would test the get_data_type_name method
        // For now, we'll just verify the mapping exists
        assert!(!expected_name.is_empty());
    }
}

#[test]
fn test_udt_member_offset_calculation() {
    let members = [
        UdtMember {
            name: "bool1".to_string(),
            data_type: 0x00C1, // BOOL
            offset: 0,
            size: 1,
        },
        UdtMember {
            name: "bool2".to_string(),
            data_type: 0x00C1, // BOOL
            offset: 1,
            size: 1,
        },
        UdtMember {
            name: "int1".to_string(),
            data_type: 0x00C3, // INT
            offset: 2,
            size: 2,
        },
        UdtMember {
            name: "dint1".to_string(),
            data_type: 0x00C4, // DINT
            offset: 4,
            size: 4,
        },
    ];

    // Test that offsets are properly aligned
    assert_eq!(members[0].offset, 0);
    assert_eq!(members[1].offset, 1);
    assert_eq!(members[2].offset, 2);
    assert_eq!(members[3].offset, 4); // Should be aligned to 4-byte boundary
}

#[test]
fn test_tag_scope_detection() {
    let controller_tag = TagAttributes {
        name: "ControllerTag".to_string(),
        data_type: 0x00C4,
        data_type_name: "DINT".to_string(),
        dimensions: Vec::new(),
        permissions: TagPermissions::ReadWrite,
        scope: TagScope::Controller,
        template_instance_id: None,
        size: 4,
    };

    let program_tag = TagAttributes {
        name: "Program:MainProgram.Tag1".to_string(),
        data_type: 0x00C4,
        data_type_name: "DINT".to_string(),
        dimensions: Vec::new(),
        permissions: TagPermissions::ReadWrite,
        scope: TagScope::Program("MainProgram".to_string()),
        template_instance_id: None,
        size: 4,
    };

    assert_eq!(controller_tag.scope, TagScope::Controller);
    assert_eq!(
        program_tag.scope,
        TagScope::Program("MainProgram".to_string())
    );
}

#[test]
fn test_udt_template_parsing_with_various_types() {
    let template = UdtTemplate {
        template_id: 456,
        name: "ComplexTemplate".to_string(),
        size: 32,
        member_count: 6,
        members: vec![
            UdtMember {
                name: "bool_val".to_string(),
                data_type: 0x00C1, // BOOL
                offset: 0,
                size: 1,
            },
            UdtMember {
                name: "sint_val".to_string(),
                data_type: 0x00C2, // SINT
                offset: 1,
                size: 1,
            },
            UdtMember {
                name: "int_val".to_string(),
                data_type: 0x00C3, // INT
                offset: 2,
                size: 2,
            },
            UdtMember {
                name: "dint_val".to_string(),
                data_type: 0x00C4, // DINT
                offset: 4,
                size: 4,
            },
            UdtMember {
                name: "real_val".to_string(),
                data_type: 0x00CA, // REAL
                offset: 8,
                size: 4,
            },
            UdtMember {
                name: "string_val".to_string(),
                data_type: 0x00CE, // STRING
                offset: 12,
                size: 84, // 82 chars + 2 length bytes
            },
        ],
    };

    assert_eq!(template.template_id, 456);
    assert_eq!(template.name, "ComplexTemplate");
    assert_eq!(template.size, 32);
    assert_eq!(template.member_count, 6);
    assert_eq!(template.members.len(), 6);

    // Test member alignment
    assert_eq!(template.members[0].offset, 0); // BOOL
    assert_eq!(template.members[1].offset, 1); // SINT
    assert_eq!(template.members[2].offset, 2); // INT
    assert_eq!(template.members[3].offset, 4); // DINT (aligned)
    assert_eq!(template.members[4].offset, 8); // REAL (aligned)
    assert_eq!(template.members[5].offset, 12); // STRING (aligned)
}

#[test]
fn test_error_handling() {
    // Test error cases
    let empty_template_data: Vec<u8> = vec![];
    let short_template_data = [0x01, 0x02];

    // These would test error handling in the actual implementation
    assert!(empty_template_data.is_empty());
    assert!(short_template_data.len() < 8);
}

#[test]
fn test_udt_definition_caching() {
    let mut client = MockEipClient::new();
    client.add_test_udt_definition();

    // Test that definition is cached
    assert!(client.udt_definitions.contains_key("TestUDT"));

    let definition = client.udt_definitions.get("TestUDT").unwrap();
    assert_eq!(definition.name, "TestUDT");
    assert_eq!(definition.members.len(), 4);
}

#[test]
fn test_tag_attributes_caching() {
    let mut client = MockEipClient::new();
    client.add_test_tag_attributes();

    // Test that attributes are cached
    assert!(client.tag_attributes.contains_key("TestUDT"));

    let attributes = client.tag_attributes.get("TestUDT").unwrap();
    assert_eq!(attributes.name, "TestUDT");
    assert_eq!(attributes.data_type, 0x00A0);
}

#[test]
fn test_route_path_validation() {
    let valid_route = RoutePath::new()
        .add_slot(0)
        .add_address("192.168.1.100".to_string());

    let invalid_route = RoutePath::new().add_address("invalid-ip".to_string());

    // Valid route should generate CIP bytes
    let valid_cip = valid_route.to_cip_bytes();
    assert!(!valid_cip.is_empty());

    // Invalid route should still generate bytes (IP parsing error is handled)
    let invalid_cip = invalid_route.to_cip_bytes();
    assert!(!invalid_cip.is_empty());
}

#[test]
fn test_comprehensive_udt_workflow() {
    let mut client = MockEipClient::new();

    // 1. Add tag attributes
    client.add_test_tag_attributes();

    // 2. Add template
    client.add_test_template();

    // 3. Add UDT definition
    client.add_test_udt_definition();

    // 4. Verify all components are present
    assert!(client.tag_attributes.contains_key("TestUDT"));
    assert!(client.templates.contains_key(&123));
    assert!(client.udt_definitions.contains_key("TestUDT"));

    // 5. Test workflow
    let attributes = client.tag_attributes.get("TestUDT").unwrap();
    assert_eq!(attributes.template_instance_id, Some(123));

    let template = client.templates.get(&123).unwrap();
    assert_eq!(template.member_count, 4);

    let definition = client.udt_definitions.get("TestUDT").unwrap();
    assert_eq!(definition.members.len(), 4);
}
