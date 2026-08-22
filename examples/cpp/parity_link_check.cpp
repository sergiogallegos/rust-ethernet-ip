#include "rust_ethernet_ip.h"

template <typename T>
void use_symbol(T symbol)
{
    (void)symbol;
}

int main()
{
    use_symbol(&eip_abi_version);
    use_symbol(&eip_library_version);
    use_symbol(&eip_capabilities);
    use_symbol(&eip_get_last_error);
    use_symbol(&eip_connect);
    use_symbol(&eip_connect_with_route);
    use_symbol(&eip_connect_with_route_hops);
    use_symbol(&eip_set_route_path);
    use_symbol(&eip_set_route_path_hops);
    use_symbol(&eip_disconnect);
    use_symbol(&eip_read_bool);
    use_symbol(&eip_write_bool);
    use_symbol(&eip_read_sint);
    use_symbol(&eip_write_sint);
    use_symbol(&eip_read_int);
    use_symbol(&eip_write_int);
    use_symbol(&eip_read_dint);
    use_symbol(&eip_write_dint);
    use_symbol(&eip_read_lint);
    use_symbol(&eip_write_lint);
    use_symbol(&eip_read_usint);
    use_symbol(&eip_write_usint);
    use_symbol(&eip_read_uint);
    use_symbol(&eip_write_uint);
    use_symbol(&eip_read_udint);
    use_symbol(&eip_write_udint);
    use_symbol(&eip_read_ulint);
    use_symbol(&eip_write_ulint);
    use_symbol(&eip_read_real);
    use_symbol(&eip_write_real);
    use_symbol(&eip_read_lreal);
    use_symbol(&eip_write_lreal);
    use_symbol(&eip_read_string);
    use_symbol(&eip_write_string);
    use_symbol(&eip_read_tag);
    use_symbol(&eip_read_array_range);
    use_symbol(&eip_read_udt);
    use_symbol(&eip_write_udt);
    use_symbol(&eip_read_udt_chunked);
    use_symbol(&eip_read_udt_member_by_offset);
    use_symbol(&eip_write_udt_member_by_offset);
    use_symbol(&eip_discover_tags);
    use_symbol(&eip_get_tag_metadata);
    use_symbol(&eip_get_udt_definition_by_id);
    use_symbol(&eip_get_tag_attributes_by_id);
    use_symbol(&eip_discover_tags_detailed_by_id);
    use_symbol(&eip_free_udt_definition);
    use_symbol(&eip_free_tag_attributes_result);
    use_symbol(&eip_free_tag_discovery_result);
    use_symbol(&eip_free_string);
    use_symbol(&eip_set_max_packet_size);
    use_symbol(&eip_check_health);
    use_symbol(&eip_check_health_detailed);
    use_symbol(&eip_get_diagnostics_json);
    use_symbol(&eip_refresh_schema);
    use_symbol(&eip_read_tags_batch);
    use_symbol(&eip_write_tags_batch);
    use_symbol(&eip_execute_batch);
    use_symbol(&eip_configure_batch_operations);
    use_symbol(&eip_get_batch_config);
    return 0;
}
