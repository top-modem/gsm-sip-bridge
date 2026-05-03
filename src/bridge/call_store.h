#pragma once

#include <cstdint>
#include <string>
#include <vector>

struct sqlite3;

struct CallRecord {
    int64_t id = 0;
    std::string module_id;
    std::string caller_id;
    std::string started_at;
    double duration_seconds = 0.0;
    std::string status;
    std::string sip_destination;
};

class CallStore {
public:
    CallStore() = default;
    ~CallStore();

    CallStore(const CallStore&) = delete;
    CallStore& operator=(const CallStore&) = delete;

    bool open(const std::string& db_path);
    void close();

    int64_t insert(const CallRecord& record);
    std::vector<CallRecord> fetch_recent(int limit = 50);
    int64_t count();

private:
    sqlite3* db_ = nullptr;

    bool create_schema();
};
