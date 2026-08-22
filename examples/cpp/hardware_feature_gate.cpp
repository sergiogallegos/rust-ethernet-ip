// Restore-safe companion gate for batch, whole-UDT, and discovery coverage.

#include "rust_ethernet_ip.h"

#include <array>
#include <cstdint>
#include <cstdlib>
#include <iostream>
#include <map>
#include <sstream>
#include <stdexcept>
#include <string>
#include <vector>

namespace {

const std::vector<std::string> batch_read_tags = {
    "gTestArray_DINT[0]",
    "gTestArray_INT[0]",
    "gTestArray_REAL[0]",
    "gTestArray_BOOL[0]",
    "gTest_STRING",
    "gTestUDT.Member1_DINT",
    "Program:TestProgram.gTestArray_DINT[0]",
    "Program:TestProgram.gTestUDT.Member1_DINT",
    "Program:TestProgram.gTestArray_REAL[0]",
    "Program:TestProgram.gTestArray_BOOL[0]",
};

const std::vector<std::string> batch_write_tags = {
    "gTestArray_DINT[50]",
    "gTestArray_DINT[51]",
    "Program:TestProgram.gTestArray_DINT[50]",
    "Program:TestProgram.gTestArray_DINT[51]",
};

const std::vector<std::string> whole_udt_tags = {
    "gTestUDT",
    "gTestUDT_Array[0]",
    "Program:TestProgram.gTestUDT",
    "Program:TestProgram.gTestUDT_Array[0]",
};

struct Options {
    std::string address = "192.168.0.1:44818";
    int slot = 0;
    std::string program = "TestProgram";
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

std::string with_program(std::string tag, const std::string &program)
{
    const std::string marker = "TestProgram";
    const std::size_t position = tag.find(marker);
    if (position != std::string::npos)
        tag.replace(position, marker.size(), program);
    return tag;
}

int32_t exercise_value(int32_t original)
{
    return original == 123456789 ? 123456788 : 123456789;
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

std::string make_write_payload(const std::map<std::string, int32_t> &values)
{
    std::ostringstream payload;
    payload << '[';
    bool first = true;
    for (const auto &entry : values) {
        if (!first) payload << ',';
        first = false;
        payload << "{\"tag_name\":\"" << entry.first
                << "\",\"value_type\":\"DINT\",\"value\":" << entry.second << '}';
    }
    payload << ']';
    return payload.str();
}

void require_batch_success(const std::string &response, std::size_t expected, const std::string &operation)
{
    if (response.find("\"success\":false") != std::string::npos)
        throw std::runtime_error(operation + " returned a per-tag failure: " + response);
    std::size_t successes = 0;
    std::size_t position = 0;
    while ((position = response.find("\"success\":true", position)) != std::string::npos) {
        ++successes;
        position += 14;
    }
    if (successes != expected) {
        throw std::runtime_error(
            operation + " returned " + std::to_string(successes) + "/" +
            std::to_string(expected) + " successful results: " + response);
    }
}

void batch_write(int client_id, const std::map<std::string, int32_t> &values)
{
    const std::string payload = make_write_payload(values);
    std::array<char, 65536> response {};
    require_ok(
        client_id,
        eip_write_tags_batch(
            client_id,
            payload.c_str(),
            static_cast<int>(values.size()),
            response.data(),
            static_cast<int>(response.size())),
        "batch write");
    require_batch_success(response.data(), values.size(), "batch write");
}

void verify_dints(int client_id, const std::map<std::string, int32_t> &expected)
{
    for (const auto &entry : expected) {
        int value = 0;
        require_ok(client_id, eip_read_dint(client_id, entry.first.c_str(), &value), "verify " + entry.first);
        if (value != entry.second) {
            throw std::runtime_error(
                entry.first + ": expected " + std::to_string(entry.second) +
                ", received " + std::to_string(value));
        }
    }
}

} // namespace

int main(int argc, char **argv)
{
    int client_id = -1;
    try {
        const Options options = parse_options(argc, argv);
        std::cout << "C/C++ companion feature gate\n"
                  << "target=" << options.address << " slot=" << options.slot
                  << " program=" << options.program << '\n'
                  << "writes are limited to four DINT test elements and are restored\n";

        if (options.dry_run) {
            std::cout << "would-test binding=cpp batch_read=" << batch_read_tags.size()
                      << " batch_write=" << batch_write_tags.size()
                      << " whole_udt=" << whole_udt_tags.size()
                      << " controller_discovery=yes program_discovery=N/A restore=yes\n";
            return 0;
        }
        if (!options.allow_writes) {
            throw std::runtime_error(
                "live mode requires --allow-writes; four dedicated DINT test elements will be changed and restored");
        }

        const uint8_t slots[] = {static_cast<uint8_t>(options.slot)};
        client_id = eip_connect_with_route(options.address.c_str(), slots, 1, nullptr, 0, nullptr, 0);
        if (client_id <= 0)
            throw std::runtime_error("connection failed to " + options.address);

        std::cout << "Phase 1 — controller discovery\n";
        EipTagDiscoveryResult discovery {};
        const int discovery_result = eip_discover_tags_detailed_by_id(client_id, &discovery);
        bool found_test_udt = false;
        if (discovery_result == 0 && discovery.success) {
            for (int index = 0; index < discovery.tag_count; ++index) {
                const char *name = discovery.tags[index].name;
                if (name != nullptr && std::string(name) == "gTestUDT") found_test_udt = true;
            }
        }
        const int discovered_count = discovery.tag_count;
        const std::string discovery_error = discovery.error_message != nullptr
            ? discovery.error_message
            : last_error(client_id);
        eip_free_tag_discovery_result(&discovery);
        if (discovery_result != 0 || !found_test_udt)
            throw std::runtime_error("controller discovery failed or omitted gTestUDT: " + discovery_error);
        std::cout << "  PASS: " << discovered_count << " controller tags\n";
        std::cout << "Phase 2 — program discovery: N/A (not exposed by the C ABI in 1.2.x)\n";

        std::cout << "Phase 3 — whole UDT reads\n";
        for (const auto &template_tag : whole_udt_tags) {
            const std::string tag = with_program(template_tag, options.program);
            std::array<char, 131072> response {};
            require_ok(
                client_id,
                eip_read_udt_chunked(client_id, tag.c_str(), response.data(), static_cast<int>(response.size())),
                "whole UDT read " + tag);
            if (response[0] == '\0') throw std::runtime_error(tag + ": empty UDT response");
            std::cout << "  PASS: " << tag << '\n';
        }

        std::cout << "Phase 4 — mixed-type batch read\n";
        std::vector<std::string> read_tags;
        std::vector<const char *> read_pointers;
        for (const auto &template_tag : batch_read_tags)
            read_tags.push_back(with_program(template_tag, options.program));
        for (const auto &tag : read_tags) read_pointers.push_back(tag.c_str());
        std::array<char, 131072> read_response {};
        require_ok(
            client_id,
            eip_read_tags_batch(
                client_id,
                read_pointers.data(),
                static_cast<int>(read_pointers.size()),
                read_response.data(),
                static_cast<int>(read_response.size())),
            "batch read");
        require_batch_success(read_response.data(), read_tags.size(), "batch read");
        std::cout << "  PASS: " << read_tags.size() << '/' << read_tags.size() << '\n';

        std::cout << "Phase 5 — restore-safe batch write\n";
        std::map<std::string, int32_t> originals;
        for (const auto &template_tag : batch_write_tags) {
            const std::string tag = with_program(template_tag, options.program);
            int value = 0;
            require_ok(client_id, eip_read_dint(client_id, tag.c_str(), &value), "read original " + tag);
            originals.emplace(tag, value);
        }
        std::map<std::string, int32_t> exercise;
        for (const auto &entry : originals) exercise.emplace(entry.first, exercise_value(entry.second));

        std::string exercise_error;
        try {
            batch_write(client_id, exercise);
            verify_dints(client_id, exercise);
        } catch (const std::exception &error) {
            exercise_error = error.what();
        }

        try {
            batch_write(client_id, originals);
            verify_dints(client_id, originals);
            std::cout << "  RESTORED: " << originals.size() << '/' << originals.size() << '\n';
        } catch (const std::exception &error) {
            throw std::runtime_error(std::string("RESTORE FAILED: ") + error.what());
        }
        if (!exercise_error.empty()) throw std::runtime_error(exercise_error);
        std::cout << "  PASS: batch write and read-back\n";

        require_ok(client_id, eip_disconnect(client_id), "disconnect");
        client_id = -1;
        std::cout << "PASS: C/C++ companion gate completed with original write values restored\n";
        return 0;
    } catch (const std::exception &error) {
        std::cerr << "FAIL: " << error.what() << '\n';
        if (client_id > 0) eip_disconnect(client_id);
        return 1;
    }
}
