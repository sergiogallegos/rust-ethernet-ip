#ifndef RUST_ETHERNET_IP_H
#define RUST_ETHERNET_IP_H

#include <stdbool.h>
#include <stdint.h>

#if defined(_WIN32)
#define EIP_API __declspec(dllimport)
#else
#define EIP_API
#endif

#ifdef __cplusplus
extern "C" {
#endif

#define RUST_ETHERNET_IP_ABI_VERSION 3u
#define RUST_ETHERNET_IP_CAP_ROUTE_PATH_ORDERED_HOPS 0x0000000000000001ull
#define RUST_ETHERNET_IP_CAP_BATCH_EXECUTE_V1 0x0000000000000002ull
#define RUST_ETHERNET_IP_CAP_DIAGNOSTICS_JSON 0x0000000000000004ull
#define RUST_ETHERNET_IP_CAP_TAG_GROUP_SUBSCRIPTIONS 0x0000000000000008ull
#define RUST_ETHERNET_IP_CAP_LAST_ERROR 0x0000000000000010ull
#define RUST_ETHERNET_IP_CAP_SCHEMA_REFRESH 0x0000000000000020ull

typedef struct EipUdtMember {
    char *name;
    int16_t data_type;
    int32_t offset;
    int32_t size;
} EipUdtMember;

typedef struct EipUdtDefinitionResult {
    bool success;
    char *error_message;
    char *name;
    EipUdtMember *members;
    int32_t member_count;
} EipUdtDefinitionResult;

typedef struct EipTagAttributes {
    char *name;
    char *data_type_name;
    int16_t data_type;
    int32_t size;
    int32_t template_instance_id;
} EipTagAttributes;

typedef struct EipTagAttributesResult {
    bool success;
    char *error_message;
    char *name;
    char *data_type_name;
    int16_t data_type;
    int32_t size;
    int32_t template_instance_id;
} EipTagAttributesResult;

typedef struct EipTagDiscoveryResult {
    bool success;
    char *error_message;
    EipTagAttributes *tags;
    int32_t tag_count;
} EipTagDiscoveryResult;

/* Runtime metadata. Returned strings are process-lifetime static unless noted. */
EIP_API uint32_t eip_abi_version(void);
EIP_API const char *eip_library_version(void);
EIP_API uint64_t eip_capabilities(void);

/* Last-error text. Returns bytes written excluding NUL, 0 for no error, or -1. */
EIP_API int eip_get_last_error(int client_id, char *buffer, int max_len);

/* Connection and routing. Successful connect calls return a positive client id. */
EIP_API int eip_connect(const char *ip_address);
EIP_API int eip_connect_with_route(
    const char *ip_address,
    const uint8_t *slots,
    int slot_count,
    const uint8_t *ports,
    int port_count,
    const char **addresses,
    int address_count);
EIP_API int eip_connect_with_route_hops(
    const char *ip_address,
    const uint8_t *hop_types,
    const uint8_t *ports,
    const uint8_t *slots,
    const char **addresses,
    int hop_count);
EIP_API int eip_set_route_path(
    int client_id,
    const uint8_t *slots,
    int slot_count,
    const uint8_t *ports,
    int port_count,
    const char **addresses,
    int address_count);
EIP_API int eip_set_route_path_hops(
    int client_id,
    const uint8_t *hop_types,
    const uint8_t *ports,
    const uint8_t *slots,
    const char **addresses,
    int hop_count);
EIP_API int eip_disconnect(int client_id);

/* Scalar tag reads and writes. Functions return 0 on success and -1 on failure. */
EIP_API int eip_read_bool(int client_id, const char *tag_name, int *result);
EIP_API int eip_write_bool(int client_id, const char *tag_name, int value);
EIP_API int eip_read_sint(int client_id, const char *tag_name, int8_t *result);
EIP_API int eip_write_sint(int client_id, const char *tag_name, int8_t value);
EIP_API int eip_read_int(int client_id, const char *tag_name, int16_t *result);
EIP_API int eip_write_int(int client_id, const char *tag_name, int16_t value);
EIP_API int eip_read_dint(int client_id, const char *tag_name, int *result);
EIP_API int eip_write_dint(int client_id, const char *tag_name, int value);
EIP_API int eip_read_lint(int client_id, const char *tag_name, int64_t *result);
EIP_API int eip_write_lint(int client_id, const char *tag_name, int64_t value);
EIP_API int eip_read_usint(int client_id, const char *tag_name, uint8_t *result);
EIP_API int eip_write_usint(int client_id, const char *tag_name, uint8_t value);
EIP_API int eip_read_uint(int client_id, const char *tag_name, uint16_t *result);
EIP_API int eip_write_uint(int client_id, const char *tag_name, uint16_t value);
EIP_API int eip_read_udint(int client_id, const char *tag_name, uint32_t *result);
EIP_API int eip_write_udint(int client_id, const char *tag_name, uint32_t value);
EIP_API int eip_read_ulint(int client_id, const char *tag_name, uint64_t *result);
EIP_API int eip_write_ulint(int client_id, const char *tag_name, uint64_t value);
EIP_API int eip_read_real(int client_id, const char *tag_name, double *result);
EIP_API int eip_write_real(int client_id, const char *tag_name, double value);
EIP_API int eip_read_lreal(int client_id, const char *tag_name, double *result);
EIP_API int eip_write_lreal(int client_id, const char *tag_name, double value);

/* STRING and generic JSON reads/writes use caller-owned UTF-8 buffers. */
EIP_API int eip_read_string(int client_id, const char *tag_name, char *result, int max_length);
EIP_API int eip_write_string(int client_id, const char *tag_name, const char *value);
EIP_API int eip_read_tag(int client_id, const char *tag_name, char *result, int max_size);
EIP_API int eip_read_array_range(
    int client_id,
    const char *base_array_name,
    int start_index,
    int element_count,
    char *result,
    int max_size);

/* UDT JSON functions use {"symbol_id":N,"data":[...]} or member-map payloads. */
EIP_API int eip_read_udt(int client_id, const char *tag_name, char *result, int max_size);
EIP_API int eip_write_udt(int client_id, const char *tag_name, const char *value, int size);
EIP_API int eip_read_udt_chunked(int client_id, const char *tag_name, char *result, int max_size);
EIP_API int eip_read_udt_member_by_offset(
    int client_id,
    const char *udt_name,
    int member_offset,
    int member_size,
    int16_t data_type,
    char *result,
    int max_size);
EIP_API int eip_write_udt_member_by_offset(
    int client_id,
    const char *udt_name,
    int member_offset,
    int member_size,
    int16_t data_type,
    const char *value,
    int size);

/* Discovery and metadata. Returned owned fields must be freed with matching free calls. */
EIP_API int eip_discover_tags(int client_id);
EIP_API int eip_get_tag_metadata(int client_id, const char *tag_name, void *metadata);
EIP_API int eip_get_udt_definition_by_id(
    int client_id,
    const char *udt_name,
    EipUdtDefinitionResult *result_ptr);
EIP_API int eip_get_tag_attributes_by_id(
    int client_id,
    const char *tag_name,
    EipTagAttributesResult *result_ptr);
EIP_API int eip_discover_tags_detailed_by_id(
    int client_id,
    EipTagDiscoveryResult *result_ptr);
EIP_API void eip_free_udt_definition(EipUdtDefinitionResult *result_ptr);
EIP_API void eip_free_tag_attributes_result(EipTagAttributesResult *result_ptr);
EIP_API void eip_free_tag_discovery_result(EipTagDiscoveryResult *result_ptr);
EIP_API void eip_free_string(char *ptr);

/* Health, diagnostics, batch, and legacy batch-config placeholders. */
EIP_API int eip_set_max_packet_size(int client_id, int size);
EIP_API int eip_check_health(int client_id, int *is_healthy);
EIP_API int eip_check_health_detailed(int client_id, int *is_healthy);
EIP_API int eip_get_diagnostics_json(int client_id, int detailed, char **result_ptr);
/* Pause writes before a controller edit/download; refresh before resuming. */
EIP_API int eip_refresh_schema(int client_id);
EIP_API int eip_read_tags_batch(
    int client_id,
    const char **tag_names,
    int tag_count,
    char *results,
    int results_capacity);
EIP_API int eip_write_tags_batch(
    int client_id,
    const char *tag_values,
    int tag_count,
    char *results,
    int results_capacity);
EIP_API int eip_execute_batch(
    int client_id,
    const char *operations,
    int operation_count,
    char *results,
    int results_capacity);
EIP_API int eip_configure_batch_operations(int client_id, const uint8_t *config);
EIP_API int eip_get_batch_config(int client_id, uint8_t *config);

#ifdef __cplusplus
}
#endif

#endif
