#pragma once

#include <atomic>
#include <ctime>
#include <chrono>
#include <mutex>
#include <thread>

class RukoServer;

class SaveScheduler {
public:
    SaveScheduler(int maxDeltaWrites, std::time_t maxDeltaTime, RukoServer &server) :
    maxDeltaWrites(maxDeltaWrites), maxDeltaTime(maxDeltaTime), lastSave(getTime()), server(server), thread(&SaveScheduler::run, this) {}

    void run();
    void registerWrite();
    void shutdown();

private:
    time_t getTime() {
        return std::time(nullptr);
    }
    void sleep(int ms) {
        std::this_thread::sleep_for(std::chrono::milliseconds(ms));
    }

    std::atomic<std::time_t> lastSave{0};
    std::atomic<int> writesSinceSave{0};
    const int maxDeltaWrites;
    const std::time_t maxDeltaTime;
    RukoServer &server;
    std::thread thread;

    std::atomic_bool isAlive{true};
    std::mutex updateEvent;
};
