#include <gtest/gtest.h>
#include "bridge/call_store.h"

#include <cstdio>

class CallStoreTest : public ::testing::Test {
protected:
    void SetUp() override {
        db_path_ = "/tmp/test_call_store.db";
        std::remove(db_path_);
        ASSERT_TRUE(store_.open(db_path_));
    }

    void TearDown() override {
        store_.close();
        std::remove(db_path_);
    }

    const char* db_path_;
    CallStore store_;
};

TEST_F(CallStoreTest, open_creates_database) {
    // Assert
    EXPECT_EQ(store_.count(), 0);
}

TEST_F(CallStoreTest, insert_returns_positive_id) {
    // Arrange
    CallRecord rec;
    rec.module_id = "ec20-abc123";
    rec.caller_id = "+1234567890";
    rec.started_at = "2026-05-03T20:00:00Z";
    rec.duration_seconds = 42.5;
    rec.status = "answered";
    rec.sip_destination = "599";

    // Act
    int64_t id = store_.insert(rec);

    // Assert
    EXPECT_GT(id, 0);
    EXPECT_EQ(store_.count(), 1);
}

TEST_F(CallStoreTest, insert_multiple_records) {
    // Arrange / Act
    for (int i = 0; i < 5; ++i) {
        CallRecord rec;
        rec.module_id = "ec20-mod" + std::to_string(i);
        rec.caller_id = "+100000000" + std::to_string(i);
        rec.started_at = "2026-05-03T20:0" + std::to_string(i) + ":00Z";
        rec.duration_seconds = static_cast<double>(i * 10);
        rec.status = (i % 2 == 0) ? "answered" : "missed";
        store_.insert(rec);
    }

    // Assert
    EXPECT_EQ(store_.count(), 5);
}

TEST_F(CallStoreTest, fetch_recent_returns_newest_first) {
    // Arrange
    for (int i = 0; i < 3; ++i) {
        CallRecord rec;
        rec.module_id = "ec20-mod";
        rec.caller_id = "+10000" + std::to_string(i);
        rec.started_at = "2026-05-03T20:0" + std::to_string(i) + ":00Z";
        rec.status = "answered";
        rec.duration_seconds = 10.0 * (i + 1);
        store_.insert(rec);
    }

    // Act
    auto results = store_.fetch_recent(10);

    // Assert
    ASSERT_EQ(results.size(), 3u);
    EXPECT_EQ(results[0].caller_id, "+100002");
    EXPECT_EQ(results[1].caller_id, "+100001");
    EXPECT_EQ(results[2].caller_id, "+100000");
}

TEST_F(CallStoreTest, fetch_recent_respects_limit) {
    // Arrange
    for (int i = 0; i < 10; ++i) {
        CallRecord rec;
        rec.module_id = "ec20-mod";
        rec.caller_id = "+1" + std::to_string(i);
        rec.started_at = "2026-05-03T20:00:00Z";
        rec.status = "answered";
        store_.insert(rec);
    }

    // Act
    auto results = store_.fetch_recent(3);

    // Assert
    EXPECT_EQ(results.size(), 3u);
}

TEST_F(CallStoreTest, insert_missed_call_with_no_duration) {
    // Arrange
    CallRecord rec;
    rec.module_id = "ec20-abc";
    rec.caller_id = "+9876543210";
    rec.started_at = "2026-05-03T21:00:00Z";
    rec.status = "missed";

    // Act
    int64_t id = store_.insert(rec);

    // Assert
    EXPECT_GT(id, 0);
    auto results = store_.fetch_recent(1);
    ASSERT_EQ(results.size(), 1u);
    EXPECT_EQ(results[0].status, "missed");
    EXPECT_DOUBLE_EQ(results[0].duration_seconds, 0.0);
    EXPECT_EQ(results[0].caller_id, "+9876543210");
}

TEST_F(CallStoreTest, insert_preserves_all_fields) {
    // Arrange
    CallRecord rec;
    rec.module_id = "ec20-XYZ";
    rec.caller_id = "+4412345678";
    rec.started_at = "2026-05-03T22:15:30Z";
    rec.duration_seconds = 123.456;
    rec.status = "answered";
    rec.sip_destination = "1001";

    // Act
    store_.insert(rec);
    auto results = store_.fetch_recent(1);

    // Assert
    ASSERT_EQ(results.size(), 1u);
    EXPECT_EQ(results[0].module_id, "ec20-XYZ");
    EXPECT_EQ(results[0].caller_id, "+4412345678");
    EXPECT_EQ(results[0].started_at, "2026-05-03T22:15:30Z");
    EXPECT_NEAR(results[0].duration_seconds, 123.456, 0.001);
    EXPECT_EQ(results[0].status, "answered");
    EXPECT_EQ(results[0].sip_destination, "1001");
}

TEST_F(CallStoreTest, insert_with_empty_caller_id) {
    // Arrange
    CallRecord rec;
    rec.module_id = "ec20-mod";
    rec.started_at = "2026-05-03T20:00:00Z";
    rec.status = "missed";

    // Act
    int64_t id = store_.insert(rec);

    // Assert
    EXPECT_GT(id, 0);
    auto results = store_.fetch_recent(1);
    ASSERT_EQ(results.size(), 1u);
    EXPECT_TRUE(results[0].caller_id.empty());
}

TEST_F(CallStoreTest, operations_on_closed_store) {
    // Arrange
    store_.close();

    // Act / Assert
    CallRecord rec;
    rec.module_id = "ec20-mod";
    rec.started_at = "2026-05-03T20:00:00Z";
    rec.status = "missed";

    EXPECT_EQ(store_.insert(rec), -1);
    EXPECT_EQ(store_.count(), 0);
    EXPECT_TRUE(store_.fetch_recent().empty());
}
