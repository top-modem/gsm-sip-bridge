#include "bridge/call_store.h"
#include "logger.h"

#include <sqlite3.h>

static constexpr const char* SCHEMA_SQL = R"(
CREATE TABLE IF NOT EXISTS calls (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    module_id        TEXT    NOT NULL,
    caller_id        TEXT    NOT NULL DEFAULT '',
    started_at       TEXT    NOT NULL,
    duration_seconds REAL    NOT NULL DEFAULT 0.0,
    status           TEXT    NOT NULL,
    sip_destination  TEXT    NOT NULL DEFAULT ''
);
CREATE INDEX IF NOT EXISTS idx_calls_started_at ON calls(started_at);
CREATE INDEX IF NOT EXISTS idx_calls_module_id ON calls(module_id);
)";

CallStore::~CallStore() {
    close();
}

bool CallStore::open(const std::string& db_path) {
    if (db_) return true;

    int rc = sqlite3_open(db_path.c_str(), &db_);
    if (rc != SQLITE_OK) {
        LOG_ERROR("call_store: cannot open database %s: %s",
                  db_path.c_str(), sqlite3_errmsg(db_));
        sqlite3_close(db_);
        db_ = nullptr;
        return false;
    }

    sqlite3_exec(db_, "PRAGMA journal_mode=WAL;", nullptr, nullptr, nullptr);
    sqlite3_exec(db_, "PRAGMA synchronous=NORMAL;", nullptr, nullptr, nullptr);

    if (!create_schema()) {
        close();
        return false;
    }

    LOG_INFO("call_store: opened %s", db_path.c_str());
    return true;
}

void CallStore::close() {
    if (db_) {
        sqlite3_close(db_);
        db_ = nullptr;
    }
}

bool CallStore::create_schema() {
    char* errmsg = nullptr;
    int rc = sqlite3_exec(db_, SCHEMA_SQL, nullptr, nullptr, &errmsg);
    if (rc != SQLITE_OK) {
        LOG_ERROR("call_store: schema creation failed: %s", errmsg ? errmsg : "unknown");
        sqlite3_free(errmsg);
        return false;
    }
    return true;
}

int64_t CallStore::insert(const CallRecord& record) {
    if (!db_) return -1;

    static constexpr const char* SQL =
        "INSERT INTO calls (module_id, caller_id, started_at, duration_seconds, status, sip_destination) "
        "VALUES (?, ?, ?, ?, ?, ?)";

    sqlite3_stmt* stmt = nullptr;
    int rc = sqlite3_prepare_v2(db_, SQL, -1, &stmt, nullptr);
    if (rc != SQLITE_OK) {
        LOG_ERROR("call_store: insert prepare failed: %s", sqlite3_errmsg(db_));
        return -1;
    }

    sqlite3_bind_text(stmt, 1, record.module_id.c_str(), -1, SQLITE_TRANSIENT);
    sqlite3_bind_text(stmt, 2, record.caller_id.c_str(), -1, SQLITE_TRANSIENT);
    sqlite3_bind_text(stmt, 3, record.started_at.c_str(), -1, SQLITE_TRANSIENT);
    sqlite3_bind_double(stmt, 4, record.duration_seconds);
    sqlite3_bind_text(stmt, 5, record.status.c_str(), -1, SQLITE_TRANSIENT);
    sqlite3_bind_text(stmt, 6, record.sip_destination.c_str(), -1, SQLITE_TRANSIENT);

    rc = sqlite3_step(stmt);
    int64_t row_id = -1;
    if (rc == SQLITE_DONE) {
        row_id = sqlite3_last_insert_rowid(db_);
    } else {
        LOG_ERROR("call_store: insert failed: %s", sqlite3_errmsg(db_));
    }

    sqlite3_finalize(stmt);
    return row_id;
}

std::vector<CallRecord> CallStore::fetch_recent(int limit) {
    std::vector<CallRecord> results;
    if (!db_) return results;

    static constexpr const char* SQL =
        "SELECT id, module_id, caller_id, started_at, duration_seconds, status, sip_destination "
        "FROM calls ORDER BY id DESC LIMIT ?";

    sqlite3_stmt* stmt = nullptr;
    int rc = sqlite3_prepare_v2(db_, SQL, -1, &stmt, nullptr);
    if (rc != SQLITE_OK) return results;

    sqlite3_bind_int(stmt, 1, limit);

    while (sqlite3_step(stmt) == SQLITE_ROW) {
        CallRecord rec;
        rec.id = sqlite3_column_int64(stmt, 0);
        rec.module_id = reinterpret_cast<const char*>(sqlite3_column_text(stmt, 1));
        rec.caller_id = reinterpret_cast<const char*>(sqlite3_column_text(stmt, 2));
        rec.started_at = reinterpret_cast<const char*>(sqlite3_column_text(stmt, 3));
        rec.duration_seconds = sqlite3_column_double(stmt, 4);
        rec.status = reinterpret_cast<const char*>(sqlite3_column_text(stmt, 5));
        rec.sip_destination = reinterpret_cast<const char*>(sqlite3_column_text(stmt, 6));
        results.push_back(std::move(rec));
    }

    sqlite3_finalize(stmt);
    return results;
}

int64_t CallStore::count() {
    if (!db_) return 0;

    static constexpr const char* SQL = "SELECT COUNT(*) FROM calls";
    sqlite3_stmt* stmt = nullptr;
    int rc = sqlite3_prepare_v2(db_, SQL, -1, &stmt, nullptr);
    if (rc != SQLITE_OK) return 0;

    int64_t result = 0;
    if (sqlite3_step(stmt) == SQLITE_ROW) {
        result = sqlite3_column_int64(stmt, 0);
    }

    sqlite3_finalize(stmt);
    return result;
}
