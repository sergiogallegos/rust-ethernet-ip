// Live companion runner for docs/validation/SCHEMA_CHANGE_GATE.md (C/C++ leg).
//
// Automates the repeatable, non-editing steps of the schema-change
// validation procedure against a real controller: baseline capture,
// post-edit read/recovery observation, explicit refresh_schema(),
// rediscovery, and restore-safe write verification. Every Studio 5000
// action stays manual and maintainer-controlled - this tool only pauses on
// stdin between phases and never issues a schema edit itself. Mirrors
// examples/schema_change_gate_live.rs, examples/CSharpSchemaGateLive, and
// python/examples/schema_change_gate_live.py phase for phase. Uses the raw
// C ABI directly (like hardware_feature_gate.cpp) rather than
// eip_client.hpp, since the RAII wrapper does not expose bool reads/writes
// or routed connect.

#include "rust_ethernet_ip.h"

#include <array>
#include <cstdint>
#include <cstdlib>
#include <iostream>
#include <stdexcept>
#include <string>
#include <utility>
#include <vector>

namespace {

const std::array<int, 2> indices = {5, 40};

struct Options {
    std::string address = "192.168.0.1:44818";
    int slot = 0;
    std::string program = "TestProgram";
    std::string tag = "gSchemaSwap";
    bool dry_run = false;
    bool allow_writes = false;
};

std::string last_error(int client_id)
{
    std::array<char, 2048> buffer {};
    const int length = eip_get_last_error(client_id, buffer.data(), static_cast<int>(buffer.size()));
    return length > 0 ? std::string(buffer.data(), static_cast<std::size_t>(length)) : std::string {};
}

void require_ok(int client_id, int result, const std::string &operation)
{
    if (result != 0)
        throw std::runtime_error(operation + ": " + last_error(client_id));
}

Options parse_options(int argc, char **argv)
{
    Options options;
    if (const char *value = std::getenv("TEST_PLC_ADDRESS")) options.address = value;
    if (const char *value = std::getenv("TEST_PLC_SLOT")) options.slot = std::stoi(value);
    if (const char *value = std::getenv("TEST_PLC_PROGRAM")) options.program = value;

    for (int index = 1; index < argc; ++index) {
        const std::string argument = argv[index];
        if (argument == "--plc-address" && index + 1 < argc)
            options.address = argv[++index];
        else if (argument == "--plc-slot" && index + 1 < argc)
            options.slot = std::stoi(argv[++index]);
        else if (argument == "--program" && index + 1 < argc)
            options.program = argv[++index];
        else if (argument == "--tag" && index + 1 < argc)
            options.tag = argv[++index];
        else if (argument == "--dry-run")
            options.dry_run = true;
        else if (argument == "--allow-writes")
            options.allow_writes = true;
        else
            throw std::invalid_argument("unknown or incomplete argument: " + argument);
    }
    if (options.slot < 0 || options.slot > 255)
        throw std::invalid_argument("PLC slot must be between 0 and 255");
    return options;
}

// A tag-value pair carrying only the two shapes this gate swaps between.
struct SchemaValue {
    enum class Kind { Unknown, Dint, Bool } kind = Kind::Unknown;
    int32_t dint_value = 0;
    bool bool_value = false;
};

std::string describe(const SchemaValue &value)
{
    switch (value.kind) {
        case SchemaValue::Kind::Dint: return "Dint(" + std::to_string(value.dint_value) + ")";
        case SchemaValue::Kind::Bool: return std::string("Bool(") + (value.bool_value ? "true" : "false") + ")";
        default: return "Unknown";
    }
}

bool values_equal(const SchemaValue &a, const SchemaValue &b)
{
    if (a.kind != b.kind) return false;
    if (a.kind == SchemaValue::Kind::Dint) return a.dint_value == b.dint_value;
    if (a.kind == SchemaValue::Kind::Bool) return a.bool_value == b.bool_value;
    return false;
}

SchemaValue read_schema_value(int client_id, const std::string &path)
{
    std::array<char, 512> buffer {};
    require_ok(
        client_id,
        eip_read_tag(client_id, path.c_str(), buffer.data(), static_cast<int>(buffer.size())),
        "read " + path);
    const std::string json(buffer.data());

    SchemaValue value;
    if (const auto tag_position = json.find("\"Bool\""); tag_position != std::string::npos) {
        value.kind = SchemaValue::Kind::Bool;
        const auto colon = json.find(':', tag_position);
        value.bool_value = json.substr(colon + 1).find("true") != std::string::npos;
    } else if (const auto dint_position = json.find("\"Dint\""); dint_position != std::string::npos) {
        value.kind = SchemaValue::Kind::Dint;
        const auto colon = json.find(':', dint_position);
        value.dint_value = std::stoi(json.substr(colon + 1));
    } else {
        throw std::runtime_error(path + ": unexpected read payload: " + json);
    }
    return value;
}

// Produces a distinguishable probe value of the same type, for a
// restore-safe write/read-back check. Only DINT[] and packed BOOL[] are
// supported, matching the shapes this gate swaps between.
SchemaValue exercise(const SchemaValue &value)
{
    SchemaValue result = value;
    if (value.kind == SchemaValue::Kind::Dint) {
        result.dint_value = value.dint_value == 123456789 ? 123456788 : 123456789;
    } else if (value.kind == SchemaValue::Kind::Bool) {
        result.bool_value = !value.bool_value;
    } else {
        throw std::runtime_error("unsupported schema-swap element type for a write probe");
    }
    return result;
}

void write_schema_value(int client_id, const std::string &path, const SchemaValue &value)
{
    if (value.kind == SchemaValue::Kind::Dint) {
        require_ok(client_id, eip_write_dint(client_id, path.c_str(), value.dint_value), "write " + path);
    } else if (value.kind == SchemaValue::Kind::Bool) {
        require_ok(client_id, eip_write_bool(client_id, path.c_str(), value.bool_value ? 1 : 0), "write " + path);
    } else {
        throw std::runtime_error("unsupported schema-swap element type for a write: " + path);
    }
}

void write_and_verify(int client_id, const std::string &path, const SchemaValue &value)
{
    write_schema_value(client_id, path, value);
    const SchemaValue read_back = read_schema_value(client_id, path);
    if (!values_equal(read_back, value)) {
        throw std::runtime_error(
            path + ": wrote " + describe(value) + ", read back " + describe(read_back));
    }
}

struct SchemaCacheMetrics {
    uint64_t generation = 0;
    uint64_t refreshes = 0;
    uint64_t hits = 0;
    uint64_t misses = 0;
    uint64_t evictions = 0;
    uint64_t contradictions = 0;
    uint64_t recoveries_ok = 0;
    uint64_t recoveries_failed = 0;
};

uint64_t extract_u64(const std::string &json, const std::string &key)
{
    const std::string needle = "\"" + key + "\":";
    const auto position = json.find(needle);
    if (position == std::string::npos) return 0;
    return std::stoull(json.substr(position + needle.size()));
}

SchemaCacheMetrics parse_schema_cache_metrics(const std::string &full_json)
{
    const auto position = full_json.find("\"schema_cache\"");
    const std::string scoped = position == std::string::npos ? std::string {} : full_json.substr(position);

    SchemaCacheMetrics metrics;
    metrics.generation = extract_u64(scoped, "generation");
    metrics.refreshes = extract_u64(scoped, "refreshes");
    metrics.hits = extract_u64(scoped, "array_classification_hits");
    metrics.misses = extract_u64(scoped, "array_classification_misses");
    metrics.evictions = extract_u64(scoped, "array_classification_evictions");
    metrics.contradictions = extract_u64(scoped, "datatype_contradictions");
    metrics.recoveries_ok = extract_u64(scoped, "successful_read_recoveries");
    metrics.recoveries_failed = extract_u64(scoped, "failed_read_recoveries");
    return metrics;
}

SchemaCacheMetrics get_schema_cache_metrics(int client_id)
{
    char *json_ptr = nullptr;
    require_ok(client_id, eip_get_diagnostics_json(client_id, 0, &json_ptr), "get diagnostics");
    const std::string json(json_ptr != nullptr ? json_ptr : "");
    if (json_ptr != nullptr) eip_free_string(json_ptr);
    return parse_schema_cache_metrics(json);
}

void print_metrics_delta(const std::string &label, const SchemaCacheMetrics &before, const SchemaCacheMetrics &after)
{
    std::cout << "  " << label << ":\n";
    std::cout << "    generation: " << before.generation << " -> " << after.generation << " ("
              << (static_cast<int64_t>(after.generation) - static_cast<int64_t>(before.generation)) << ")\n";
    std::cout << "    refreshes: " << before.refreshes << " -> " << after.refreshes << " ("
              << (static_cast<int64_t>(after.refreshes) - static_cast<int64_t>(before.refreshes)) << ")\n";
    std::cout << "    array classification hits/misses/evictions: " << before.hits << '/' << before.misses << '/'
              << before.evictions << " -> " << after.hits << '/' << after.misses << '/' << after.evictions << '\n';
    std::cout << "    datatype contradictions: " << before.contradictions << " -> " << after.contradictions << " ("
              << (static_cast<int64_t>(after.contradictions) - static_cast<int64_t>(before.contradictions)) << ")\n";
    std::cout << "    read recoveries succeeded/failed: " << before.recoveries_ok << '/' << before.recoveries_failed
              << " -> " << after.recoveries_ok << '/' << after.recoveries_failed << '\n';
}

void pause_for_studio5000(const std::string &message)
{
    std::cout << "\n=== MAINTAINER ACTION REQUIRED ===\n"
              << message << '\n'
              << "This tool never edits controller schema. Perform the Studio 5000 action now.\n"
              << "Press Enter once the change is downloaded and online: ";
    std::cout.flush();
    std::string line;
    std::getline(std::cin, line);
}

struct ScopedPath {
    std::string scope_name;
    std::string base_path;
};

} // namespace

int main(int argc, char **argv)
{
    int client_id = -1;
    try {
        const Options options = parse_options(argc, argv);
        std::cout << "Schema-change live gate companion (C/C++)\n"
                  << "target=" << options.address << " slot=" << options.slot
                  << " program=" << options.program << " tag=" << options.tag
                  << " allow_writes=" << (options.allow_writes ? "true" : "false") << '\n'
                  << "This tool never edits controller schema; every Studio 5000 action stays manual.\n";

        if (options.dry_run) {
            std::cout << "would-test scopes=controller,program indices=[5, 40] allow_writes="
                      << (options.allow_writes ? "true" : "false") << '\n';
            return 0;
        }
        if (!options.allow_writes) {
            throw std::runtime_error(
                "live mode requires --allow-writes; dedicated gSchemaSwap elements will be changed and restored");
        }

        const uint8_t slots[] = {static_cast<uint8_t>(options.slot)};
        client_id = eip_connect_with_route(options.address.c_str(), slots, 1, nullptr, 0, nullptr, 0);
        if (client_id <= 0)
            throw std::runtime_error("connection failed to " + options.address);

        int healthy = 0;
        eip_check_health(client_id, &healthy);
        std::cout << "Phase 0 — connected; healthy=" << (healthy != 0 ? "true" : "false") << '\n';

        const std::vector<ScopedPath> scopes = {
            {"controller", options.tag},
            {"program", "Program:" + options.program + "." + options.tag},
        };

        const SchemaCacheMetrics baseline_metrics = get_schema_cache_metrics(client_id);
        std::cout << "Phase 1 — baseline schema_cache_metrics: generation=" << baseline_metrics.generation
                  << " refreshes=" << baseline_metrics.refreshes << '\n';

        std::cout << "Phase 2 — pre-edit reads (twice, to warm classification cache)\n";
        std::vector<std::pair<std::string, SchemaValue>> pre_edit_values;
        for (const auto &scope : scopes) {
            for (const int index : indices) {
                const std::string path = scope.base_path + "[" + std::to_string(index) + "]";
                const SchemaValue first = read_schema_value(client_id, path);
                const SchemaValue second = read_schema_value(client_id, path);
                if (!values_equal(first, second)) {
                    throw std::runtime_error(
                        path + ": unstable read before any edit: " + describe(first) + " then " + describe(second));
                }
                std::cout << "  " << scope.scope_name << ' ' << path << " = " << describe(second) << '\n';
                pre_edit_values.emplace_back(path, second);
            }
        }

        std::cout << "Phase 3 — restore-safe pre-edit write smoke check\n";
        for (const auto &[path, original] : pre_edit_values) {
            const SchemaValue probe = exercise(original);
            write_and_verify(client_id, path, probe);
            write_and_verify(client_id, path, original);
            std::cout << "  " << path << ": exercised and restored to " << describe(original) << '\n';
        }

        pause_for_studio5000(
            "Move any test-only references off '" + options.tag + "', delete the unused original, and rename "
            "the replacement to '" + options.tag + "' — for both controller and program scope.");

        std::cout << "Phase 4 — post-edit reads without calling refresh_schema() first\n";
        const SchemaCacheMetrics pre_refresh_metrics = get_schema_cache_metrics(client_id);
        std::vector<std::pair<std::string, SchemaValue>> post_edit_values;
        for (const auto &scope : scopes) {
            for (const int index : indices) {
                const std::string path = scope.base_path + "[" + std::to_string(index) + "]";
                try {
                    const SchemaValue value = read_schema_value(client_id, path);
                    std::cout << "  " << scope.scope_name << ' ' << path << " = " << describe(value)
                              << " (automatic recovery applies if the type changed)\n";
                    post_edit_values.emplace_back(path, value);
                } catch (const std::exception &error) {
                    std::cout << "  " << scope.scope_name << ' ' << path
                              << ": read error before refresh: " << error.what() << '\n';
                }
            }
        }
        const SchemaCacheMetrics post_read_metrics = get_schema_cache_metrics(client_id);
        print_metrics_delta("automatic recovery (no explicit refresh yet)", pre_refresh_metrics, post_read_metrics);

        std::cout << "Phase 5 — explicit refresh_schema()\n";
        require_ok(client_id, eip_refresh_schema(client_id), "refresh schema");
        const SchemaCacheMetrics post_refresh_metrics = get_schema_cache_metrics(client_id);
        if (post_refresh_metrics.generation != pre_refresh_metrics.generation + 1 ||
            post_refresh_metrics.refreshes != pre_refresh_metrics.refreshes + 1) {
            throw std::runtime_error(
                "refresh_schema() did not advance generation/refresh count by exactly one");
        }
        std::cout << "  generation now " << post_refresh_metrics.generation << '\n';

        std::cout << "Phase 6 — rediscovery\n";
        EipTagDiscoveryResult discovery {};
        const int discovery_result = eip_discover_tags_detailed_by_id(client_id, &discovery);
        int matches = 0;
        if (discovery_result == 0 && discovery.success) {
            for (int index = 0; index < discovery.tag_count; ++index) {
                const char *name = discovery.tags[index].name;
                if (name != nullptr && options.tag == name) ++matches;
            }
        }
        const int discovered_count = discovery.tag_count;
        eip_free_tag_discovery_result(&discovery);
        if (discovery_result == 0) {
            std::cout << "  controller discovery: " << discovered_count << " tags, " << matches << " match '"
                      << options.tag << "'\n";
        } else {
            std::cout << "  controller discovery failed (non-fatal): " << last_error(client_id) << '\n';
        }
        std::cout << "  program discovery: N/A (not exposed by the C ABI in 1.2.x)\n";

        std::cout << "Phase 7 — post-refresh reads\n";
        std::vector<std::pair<std::string, SchemaValue>> post_refresh_values;
        for (const auto &scope : scopes) {
            for (const int index : indices) {
                const std::string path = scope.base_path + "[" + std::to_string(index) + "]";
                const SchemaValue value = read_schema_value(client_id, path);
                std::cout << "  " << scope.scope_name << ' ' << path << " = " << describe(value) << '\n';
                post_refresh_values.emplace_back(path, value);
            }
        }

        std::cout << "Phase 8 — restore-safe post-refresh write/verify\n";
        for (const auto &[path, current] : post_refresh_values) {
            const SchemaValue probe = exercise(current);
            write_and_verify(client_id, path, probe);
            write_and_verify(client_id, path, current);
            std::cout << "  " << path << ": exercised the new addressing shape and restored to " << describe(current)
                      << '\n';
        }

        const SchemaCacheMetrics final_metrics = get_schema_cache_metrics(client_id);
        int final_healthy = 0;
        eip_check_health(client_id, &final_healthy);
        std::cout << "\n=== Paste into the dated validation record ===\n";
        std::cout << "session survived: yes (single connection held for the entire run; healthy="
                  << (final_healthy != 0 ? "true" : "false") << ")\n";
        print_metrics_delta("baseline -> final", baseline_metrics, final_metrics);
        std::cout << "C/C++: PASS\n";

        require_ok(client_id, eip_disconnect(client_id), "disconnect");
        client_id = -1;
        return 0;
    } catch (const std::exception &error) {
        std::cerr << "FAIL: " << error.what() << '\n';
        if (client_id > 0) eip_disconnect(client_id);
        return 1;
    }
}
