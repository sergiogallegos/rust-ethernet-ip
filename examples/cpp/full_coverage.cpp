// C/C++ full-coverage exerciser — parity with the Rust/C#/Python runners.
//
// Parses examples/full_coverage_tags.json directly (single source of truth),
// expands the categories with the same rules as examples/test_plc_full_coverage.rs,
// and drives the full read / write / verify / blocked-probe / settle surface
// through the C ABI in include/rust_ethernet_ip.h.
//
// Unlike the current Rust/C#/Python runners, this one DOES exercise STRING writes
// (standalone STRINGs are written+verified; UDT STRING members are blocked-probed),
// so its counts reflect the manifest labels in full: 2304 read / 2268 write /
// 17 blocked / 19 read-only.
//
// Usage: full_coverage [--plc-address <ip:port>] [--plc-slot <n>]
//                      [--manifest <path>] [--out-dir <dir>] [--skip-preflight]
// Env fallbacks: TEST_PLC_ADDRESS (default 192.168.0.1:44818), TEST_PLC_SLOT (0).

#include "rust_ethernet_ip.h"

#include <cctype>
#include <cmath>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <ctime>
#include <fstream>
#include <map>
#include <sstream>
#include <string>
#include <utility>
#include <vector>

// ------------------------------- tiny JSON ---------------------------------
struct Json {
    enum Type { Null, Bool, Num, Str, Arr, Obj } type = Null;
    bool b = false;
    double num = 0;
    std::string str;
    std::vector<Json> arr;
    std::vector<std::pair<std::string, Json>> obj; // insertion order preserved

    const Json *find(const std::string &k) const {
        if (type != Obj) return nullptr;
        for (const auto &p : obj)
            if (p.first == k) return &p.second;
        return nullptr;
    }
};

struct JsonParser {
    const char *p;
    const char *end;
    explicit JsonParser(const std::string &s) : p(s.data()), end(s.data() + s.size()) {}

    void ws() {
        while (p < end && (*p == ' ' || *p == '\t' || *p == '\n' || *p == '\r')) ++p;
    }
    Json parse() {
        ws();
        Json v = value();
        return v;
    }
    Json value() {
        ws();
        if (p >= end) return {};
        char c = *p;
        if (c == '{') return object();
        if (c == '[') return array();
        if (c == '"') { Json v; v.type = Json::Str; v.str = str(); return v; }
        if (c == 't' || c == 'f') return boolean();
        if (c == 'n') { p += 4; return {}; }
        return number();
    }
    std::string str() {
        std::string out;
        ++p; // opening quote
        while (p < end && *p != '"') {
            if (*p == '\\' && p + 1 < end) {
                ++p;
                switch (*p) {
                    case 'n': out += '\n'; break;
                    case 't': out += '\t'; break;
                    case '"': out += '"'; break;
                    case '\\': out += '\\'; break;
                    case '/': out += '/'; break;
                    default: out += *p; break;
                }
                ++p;
            } else {
                out += *p++;
            }
        }
        if (p < end) ++p; // closing quote
        return out;
    }
    Json number() {
        Json v;
        v.type = Json::Num;
        const char *start = p;
        while (p < end && (std::isdigit((unsigned char)*p) || *p == '-' || *p == '+' ||
                           *p == '.' || *p == 'e' || *p == 'E'))
            ++p;
        v.num = std::strtod(std::string(start, p).c_str(), nullptr);
        return v;
    }
    Json boolean() {
        Json v;
        v.type = Json::Bool;
        if (*p == 't') { v.b = true; p += 4; } else { v.b = false; p += 5; }
        return v;
    }
    Json array() {
        Json v;
        v.type = Json::Arr;
        ++p; // [
        ws();
        while (p < end && *p != ']') {
            v.arr.push_back(value());
            ws();
            if (p < end && *p == ',') { ++p; ws(); }
        }
        if (p < end) ++p; // ]
        return v;
    }
    Json object() {
        Json v;
        v.type = Json::Obj;
        ++p; // {
        ws();
        while (p < end && *p != '}') {
            ws();
            std::string key = str();
            ws();
            if (p < end && *p == ':') ++p;
            v.obj.emplace_back(std::move(key), value());
            ws();
            if (p < end && *p == ',') { ++p; ws(); }
        }
        if (p < end) ++p; // }
        return v;
    }
};

// ------------------------------- model -------------------------------------
enum class Kind { Dint, Int, Real, Bool, String, Udt };
enum class Write { Writeable, ReadOnly, BlockedString };

static Kind kind_of(const std::string &s) {
    if (s == "Dint") return Kind::Dint;
    if (s == "Int") return Kind::Int;
    if (s == "Real") return Kind::Real;
    if (s == "Bool") return Kind::Bool;
    if (s == "String") return Kind::String;
    return Kind::Udt;
}
static Write write_of(const std::string &s) {
    if (s == "writeable" || s == "service_layer_writeable") return Write::Writeable;
    if (s == "encoding_blocked_udt_string_member") return Write::BlockedString;
    return Write::ReadOnly;
}
static bool is_writeable(Write w) { return w == Write::Writeable; }
static bool is_blocked(Write w) { return w == Write::BlockedString; }

struct Tag {
    std::string name;
    std::string category;
    Kind kind;
    Write write;
};

static std::string render(std::string pat, long i, const std::string &member,
                          const std::string &field, long j) {
    auto replace = [&](const std::string &from, const std::string &to) {
        size_t pos;
        while ((pos = pat.find(from)) != std::string::npos) pat.replace(pos, from.size(), to);
    };
    if (i >= 0) replace("{i}", std::to_string(i));
    if (!member.empty()) replace("{member}", member);
    if (!field.empty()) replace("{field}", field);
    if (j >= 0) replace("{j}", std::to_string(j));
    return pat;
}

static std::vector<long> range_or_once(const Json *spec) {
    std::vector<long> out;
    if (spec) {
        const Json *r = spec->find("range");
        if (r && r->type == Json::Arr && r->arr.size() == 2) {
            long a = (long)r->arr[0].num, b = (long)r->arr[1].num;
            for (long i = a; i < b; ++i) out.push_back(i);
            return out;
        }
    }
    out.push_back(0);
    return out;
}

static void expand(const Json &cat, std::vector<Tag> &tags) {
    std::string name = cat.find("name")->str;
    std::string pattern = cat.find("pattern")->str;

    if (const Json *members = cat.find("members")) {
        for (long i : range_or_once(cat.find("indices"))) {
            for (const auto &m : members->obj) {
                tags.push_back({render(pattern, i, m.first, "", -1), name,
                                kind_of(m.second.find("kind")->str),
                                write_of(m.second.find("writeability")->str)});
            }
        }
        return;
    }
    if (const Json *inner = cat.find("inner")) {
        for (long i : range_or_once(cat.find("outer_indices"))) {
            for (const auto &f : inner->obj) {
                const Json *r = f.second.find("range");
                long a = (long)r->arr[0].num, b = (long)r->arr[1].num;
                for (long j = a; j < b; ++j) {
                    tags.push_back({render(pattern, i, "", f.first, j), name,
                                    kind_of(f.second.find("kind")->str),
                                    write_of(f.second.find("writeability")->str)});
                }
            }
        }
        return;
    }
    Kind k = kind_of(cat.find("kind")->str);
    Write w = write_of(cat.find("writeability")->str);
    for (long i : range_or_once(cat.find("indices")))
        tags.push_back({render(pattern, i, "", "", -1), name, k, w});
}

// ------------------------------- stats -------------------------------------
struct CatStats {
    long read_ok = 0, read_fail = 0, write_ok = 0, write_fail = 0;
    long verify_ok = 0, verify_fail = 0, blocked_ok = 0, blocked_unexpected = 0;
};

// A written value we can verify later, tagged by kind.
struct Written {
    size_t idx;
    Kind kind;
    int i = 0;
    double r = 0;
    std::string s;
};

// ------------------------------- IO helpers --------------------------------
static bool read_ok(int cid, const Tag &t) {
    char buf[8192];
    return eip_read_tag(cid, t.name.c_str(), buf, (int)sizeof(buf)) == 0;
}

int main(int argc, char **argv) {
    std::string address = std::getenv("TEST_PLC_ADDRESS") ? std::getenv("TEST_PLC_ADDRESS")
                                                           : "192.168.0.1:44818";
    int slot = std::getenv("TEST_PLC_SLOT") ? std::atoi(std::getenv("TEST_PLC_SLOT")) : 0;
    std::string manifest_path = "examples/full_coverage_tags.json";
    std::string out_dir = "examples/full_coverage_results";
    bool skip_preflight = false;
    for (int i = 1; i < argc; ++i) {
        std::string a = argv[i];
        auto next = [&]() { return i + 1 < argc ? std::string(argv[++i]) : std::string(); };
        if (a == "--plc-address") address = next();
        else if (a == "--plc-slot") slot = std::atoi(next().c_str());
        else if (a == "--manifest") manifest_path = next();
        else if (a == "--out-dir") out_dir = next();
        else if (a == "--skip-preflight") skip_preflight = true;
    }

    std::ifstream in(manifest_path);
    if (!in) {
        std::fprintf(stderr, "manifest-error: cannot open %s\n", manifest_path.c_str());
        return 2;
    }
    std::stringstream ss;
    ss << in.rdbuf();
    Json manifest = JsonParser(ss.str()).parse();
    const Json *cats = manifest.find("categories");
    if (!cats || cats->type != Json::Arr) {
        std::fprintf(stderr, "manifest-error: no categories array\n");
        return 2;
    }
    std::vector<Tag> tags;
    for (const auto &c : cats->arr) expand(c, tags);

    long writeable = 0, blocked = 0, readonly = 0;
    for (const auto &t : tags) {
        if (is_writeable(t.write)) ++writeable;
        else if (is_blocked(t.write)) ++blocked;
        else ++readonly;
    }
    std::printf("C/C++ full-coverage — %zu tags (writeable %ld, blocked %ld, read-only %ld)\n",
                tags.size(), writeable, blocked, readonly);

    const uint8_t slots[1] = {(uint8_t)slot};
    int cid = eip_connect_with_route(address.c_str(), slots, 1, nullptr, 0, nullptr, 0);
    if (cid <= 0) {
        std::fprintf(stderr, "connect failed to %s slot %d\n", address.c_str(), slot);
        return 2;
    }

    std::map<std::string, CatStats> stats;

    // Phase 0 — preflight
    if (!skip_preflight) {
        long ok = 0, fail = 0;
        for (const auto &t : tags) {
            if (read_ok(cid, t)) ++ok;
            else {
                ++fail;
                std::fprintf(stderr, "setup-error: tag %s failed preflight\n", t.name.c_str());
            }
        }
        std::printf("Phase 0 — preflight  %ld/%ld\n", ok, ok + fail);
        if (fail > 0) { eip_disconnect(cid); return 2; }
    }

    // Phase 1 — read every tag
    for (const auto &t : tags) {
        CatStats &s = stats[t.category];
        if (read_ok(cid, t)) ++s.read_ok;
        else ++s.read_fail;
    }
    std::printf("Phase 1 — read every tag: done\n");

    // Phase 2 — write random values to every writeable tag (STRINGs included)
    std::vector<Written> written;
    uint32_t rng = 0x1234567u;
    auto nextr = [&]() { rng = rng * 1664525u + 1013904223u; return rng; };
    for (size_t idx = 0; idx < tags.size(); ++idx) {
        const Tag &t = tags[idx];
        if (!is_writeable(t.write)) continue;
        CatStats &s = stats[t.category];
        int rc = -1;
        Written w{idx, t.kind};
        switch (t.kind) {
            case Kind::Dint: { w.i = (int)(nextr() % 100000); rc = eip_write_dint(cid, t.name.c_str(), w.i); break; }
            case Kind::Int:  { w.i = (int)(nextr() % 20000) - 10000; rc = eip_write_int(cid, t.name.c_str(), (int16_t)w.i); break; }
            case Kind::Bool: { w.i = (int)(nextr() & 1); rc = eip_write_bool(cid, t.name.c_str(), w.i); break; }
            case Kind::Real: { w.r = (double)(nextr() % 1000); rc = eip_write_real(cid, t.name.c_str(), w.r); break; }
            case Kind::String: { w.s = "CPPCOV" + std::to_string(idx); rc = eip_write_string(cid, t.name.c_str(), w.s.c_str()); break; }
            case Kind::Udt: continue;
        }
        if (rc == 0) { ++s.write_ok; written.push_back(w); }
        else ++s.write_fail;
    }
    std::printf("Phase 2 — write writeable: done (%zu written)\n", written.size());

    // Phase 3 — verify writes via typed read-back
    for (const auto &w : written) {
        const Tag &t = tags[w.idx];
        CatStats &s = stats[t.category];
        bool ok = false;
        switch (w.kind) {
            case Kind::Dint: { int v = 0; ok = eip_read_dint(cid, t.name.c_str(), &v) == 0 && v == w.i; break; }
            case Kind::Int:  { int16_t v = 0; ok = eip_read_int(cid, t.name.c_str(), &v) == 0 && v == (int16_t)w.i; break; }
            case Kind::Bool: { int v = 0; ok = eip_read_bool(cid, t.name.c_str(), &v) == 0 && (v != 0) == (w.i != 0); break; }
            case Kind::Real: { double v = 0; ok = eip_read_real(cid, t.name.c_str(), &v) == 0 && std::fabs(v - w.r) < 0.001; break; }
            case Kind::String: { char b[512]; ok = eip_read_string(cid, t.name.c_str(), b, (int)sizeof(b)) == 0 && w.s == b; break; }
            case Kind::Udt: break;
        }
        if (ok) ++s.verify_ok; else ++s.verify_fail;
    }
    std::printf("Phase 3 — verify: done\n");

    // Phase 4 — confirm expected-blocked writes are still rejected (STRING members)
    for (const auto &t : tags) {
        if (!is_blocked(t.write)) continue;
        CatStats &s = stats[t.category];
        int rc = eip_write_string(cid, t.name.c_str(), "x");
        if (rc != 0) ++s.blocked_ok; else ++s.blocked_unexpected;
    }
    std::printf("Phase 4 — blocked-probe: done\n");

    // Phase 5 — settle writeable tags to a terminal state
    for (size_t idx = 0; idx < tags.size(); ++idx) {
        const Tag &t = tags[idx];
        if (!is_writeable(t.write)) continue;
        switch (t.kind) {
            case Kind::Dint: eip_write_dint(cid, t.name.c_str(), 999999); break;
            case Kind::Int: eip_write_int(cid, t.name.c_str(), (int16_t)9999); break;
            case Kind::Bool: eip_write_bool(cid, t.name.c_str(), 1); break;
            case Kind::Real: eip_write_real(cid, t.name.c_str(), 99.0); break;
            case Kind::String: eip_write_string(cid, t.name.c_str(), "SETTLED"); break;
            case Kind::Udt: break;
        }
    }
    std::printf("Phase 5 — settle: done\n");

    // Phase 6 — settle-verify a sample (first writeable tag per category)
    long settle_verify_ok = 0, settle_verify_fail = 0;
    std::map<std::string, bool> sampled;
    for (size_t idx = 0; idx < tags.size(); ++idx) {
        const Tag &t = tags[idx];
        if (!is_writeable(t.write) || sampled[t.category]) continue;
        sampled[t.category] = true;
        bool ok = false;
        switch (t.kind) {
            case Kind::Dint: { int v = 0; ok = eip_read_dint(cid, t.name.c_str(), &v) == 0 && v == 999999; break; }
            case Kind::Int: { int16_t v = 0; ok = eip_read_int(cid, t.name.c_str(), &v) == 0 && v == (int16_t)9999; break; }
            case Kind::Bool: { int v = 0; ok = eip_read_bool(cid, t.name.c_str(), &v) == 0 && v != 0; break; }
            case Kind::Real: { double v = 0; ok = eip_read_real(cid, t.name.c_str(), &v) == 0 && std::fabs(v - 99.0) < 0.001; break; }
            case Kind::String: { char b[512]; ok = eip_read_string(cid, t.name.c_str(), b, (int)sizeof(b)) == 0 && std::strcmp(b, "SETTLED") == 0; break; }
            case Kind::Udt: break;
        }
        if (ok) ++settle_verify_ok; else ++settle_verify_fail;
    }
    std::printf("Phase 6 — settle-verify: %ld/%ld\n", settle_verify_ok, settle_verify_ok + settle_verify_fail);

    eip_disconnect(cid);

    // Totals + report
    CatStats tot;
    std::printf("\nPer-category results:\n");
    std::printf("  %-32s %8s %8s %8s %8s %8s %8s\n", "category", "read+", "read-", "write+",
                "write-", "verify+", "blocked+");
    for (const auto &kv : stats) {
        const CatStats &s = kv.second;
        std::printf("  %-32s %8ld %8ld %8ld %8ld %8ld %8ld\n", kv.first.c_str(), s.read_ok,
                    s.read_fail, s.write_ok, s.write_fail, s.verify_ok, s.blocked_ok);
        tot.read_ok += s.read_ok; tot.read_fail += s.read_fail;
        tot.write_ok += s.write_ok; tot.write_fail += s.write_fail;
        tot.verify_ok += s.verify_ok; tot.verify_fail += s.verify_fail;
        tot.blocked_ok += s.blocked_ok; tot.blocked_unexpected += s.blocked_unexpected;
    }
    std::printf("  %-32s %8ld %8ld %8ld %8ld %8ld %8ld\n", "TOTAL", tot.read_ok, tot.read_fail,
                tot.write_ok, tot.write_fail, tot.verify_ok, tot.blocked_ok);

    long anomalies = tot.read_fail + tot.write_fail + tot.verify_fail + tot.blocked_unexpected +
                     settle_verify_fail;
    std::printf("\nSummary: reads=%ld/%zu  writes=%ld/%ld  verify=%ld/%ld  "
                "blocked_as_expected=%ld  unexpected_anomalies=%ld\n",
                tot.read_ok, tags.size(), tot.write_ok, tot.write_ok + tot.write_fail,
                tot.verify_ok, tot.verify_ok + tot.verify_fail, tot.blocked_ok, anomalies);
    bool pass = anomalies == 0 && tot.read_ok == (long)tags.size();
    std::printf("binding=cpp tags=%zu reads=%ld/%zu writes=%ld/%ld verify=%ld/%ld blocked=%ld "
                "anomalies=%ld RESULT=%s\n",
                tags.size(), tot.read_ok, tags.size(), tot.write_ok, tot.write_ok + tot.write_fail,
                tot.verify_ok, tot.verify_ok + tot.verify_fail, tot.blocked_ok, anomalies,
                pass ? "PASS" : "FAIL");

    // JSON result artifact
    std::time_t now = std::time(nullptr);
    char fname[512];
    std::snprintf(fname, sizeof(fname), "%s/cpp_%lld.json", out_dir.c_str(), (long long)now);
    std::ofstream out(fname);
    if (out) {
        out << "{\n  \"binding\": \"cpp\",\n  \"tags\": " << tags.size()
            << ",\n  \"reads\": " << tot.read_ok << ",\n  \"writes\": " << tot.write_ok
            << ",\n  \"verify\": " << tot.verify_ok << ",\n  \"blocked_as_expected\": "
            << tot.blocked_ok << ",\n  \"unexpected_anomalies\": " << anomalies
            << ",\n  \"result\": \"" << (pass ? "PASS" : "FAIL") << "\"\n}\n";
        std::printf("wrote %s\n", fname);
    }
    std::printf("RESULT: %s\n", pass ? "PASS" : "FAIL");
    return pass ? 0 : 1;
}
