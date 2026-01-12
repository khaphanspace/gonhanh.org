#pragma once
#include <atomic>
#include <array>

// Lock-free SPSC (Single Producer Single Consumer) queue
// Thread-safe without mutexes using atomic operations
// Producer: Hook callback thread
// Consumer: Worker thread
template<typename T, size_t Size = 512>
class LockFreeQueue {
public:
    LockFreeQueue() : m_head(0), m_tail(0) {}

    // Push item to queue (called by hook callback - must be fast)
    // Returns false if queue is full
    bool Push(const T& item) {
        size_t head = m_head.load(std::memory_order_relaxed);
        size_t next = (head + 1) % Size;

        // Check if queue is full
        if (next == m_tail.load(std::memory_order_acquire)) {
            return false;  // Queue full
        }

        // Write item
        m_buffer[head] = item;

        // Update head (release ensures item write is visible to consumer)
        m_head.store(next, std::memory_order_release);
        return true;
    }

    // Pop item from queue (called by worker thread)
    // Returns false if queue is empty
    bool Pop(T& item) {
        size_t tail = m_tail.load(std::memory_order_relaxed);

        // Check if queue is empty
        if (tail == m_head.load(std::memory_order_acquire)) {
            return false;  // Queue empty
        }

        // Read item
        item = m_buffer[tail];

        // Update tail (release for consistency, not strictly needed for SPSC)
        m_tail.store((tail + 1) % Size, std::memory_order_release);
        return true;
    }

    // Check if queue is empty (approximate - not strictly synchronized)
    bool IsEmpty() const {
        return m_tail.load(std::memory_order_relaxed) ==
               m_head.load(std::memory_order_relaxed);
    }

    // Get approximate queue size (not strictly synchronized)
    size_t ApproximateSize() const {
        size_t head = m_head.load(std::memory_order_relaxed);
        size_t tail = m_tail.load(std::memory_order_relaxed);
        if (head >= tail) {
            return head - tail;
        } else {
            return Size - tail + head;
        }
    }

private:
    std::array<T, Size> m_buffer;
    alignas(64) std::atomic<size_t> m_head;  // Cache line alignment to avoid false sharing
    alignas(64) std::atomic<size_t> m_tail;
};
