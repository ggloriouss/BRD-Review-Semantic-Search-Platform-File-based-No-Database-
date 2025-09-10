// native/spfresh_c_api.cc
#include <cstdint>
#include <cstddef>
#include <new>        // ✅ ต้องมี เพื่อใช้ std::nothrow

// ---------- Types shared with Rust ----------
struct SPFreshStatus {
    int32_t code;
    const char* message; // nullptr หรือ string literal แบบ static ก็ได้
};

using SPFreshIndex = void*;

// ---------- Internal C++ types / helpers (นอก extern "C") ----------
struct SPFreshHandle {
    // ใส่ของจริงภายหลังได้ เช่น pointer ไปยัง index engine
    int32_t dim = 0;
    // TODO: เก็บ path/index params ถ้าต้องการ
};

// มาโครปิด warning "unused"
#define UNUSED(x) (void)(x)

extern "C" {

// เปิด/สร้าง index
SPFreshStatus spfresh_open(const char* index_dir,
                           int32_t dim,
                           const char* params,
                           void** out_handle) {
    // TODO: ต่อกับ SPFresh จริง ๆ ตรงนี้
    UNUSED(index_dir);
    UNUSED(params);

    if (!out_handle) {
        static const char* kMsg = "out_handle is null";
        return {1, kMsg};
    }

    // ✅ ใช้ std::nothrow ได้แล้วเพราะ include <new> แล้ว
    SPFreshHandle* h = new (std::nothrow) SPFreshHandle();
    if (!h) {
        static const char* kMsg = "alloc handle failed";
        return {2, kMsg};
    }
    h->dim = dim;
    *out_handle = reinterpret_cast<void*>(h);
    return {0, nullptr};
}

// ปิด/คืนทรัพยากร
void spfresh_close(SPFreshIndex handle) {
    if (!handle) return;
    auto* h = reinterpret_cast<SPFreshHandle*>(handle);
    delete h;
}

// เพิ่มเวกเตอร์
SPFreshStatus spfresh_add(SPFreshIndex handle,
                          const float* vectors,
                          size_t n,
                          const int64_t* ids) {
    // TODO: ใส่ลง index จริง
    UNUSED(handle);
    UNUSED(vectors);
    UNUSED(n);
    UNUSED(ids);
    return {0, nullptr};
}

// ค้นหา
SPFreshStatus spfresh_search(SPFreshIndex handle,
                             const float* query,
                             int32_t topk,
                             int64_t* out_ids,
                             float* out_scores) {
    // TODO: ค้นหาจริง แล้วเติมผลลัพธ์
    UNUSED(handle);
    UNUSED(query);

    if (topk > 0 && out_ids && out_scores) {
        for (int32_t i = 0; i < topk; ++i) {
            out_ids[i] = -1;      // ไม่มีผลจริง
            out_scores[i] = 0.0f; // คะแนนว่าง
        }
    }
    return {0, nullptr};
}

// บันทึก index
SPFreshStatus spfresh_save(SPFreshIndex handle) {
    UNUSED(handle);
    // TODO: persist index
    return {0, nullptr};
}

} // extern "C"
