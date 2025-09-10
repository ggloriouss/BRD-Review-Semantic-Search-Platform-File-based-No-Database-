#pragma once
#include <cstddef>
#include <cstdint>

#ifdef __cplusplus
extern "C" {
#endif

// handle ทึบของ index
typedef void* SPFreshIndex;

typedef struct {
    int32_t code;       // 0 = OK
    const char* message; // optional (nullptr ถ้าไม่มี)
} SPFreshStatus;

// เปิด/สร้าง index ที่เก็บเป็นไฟล์ใน index_dir
SPFreshStatus spfresh_open(const char* index_dir,
                           int32_t dim,
                           const char* params,   // path หรือ key=value ใส่พารามิเตอร์
                           SPFreshIndex* out_handle);

// ปิดทำลาย
void spfresh_close(SPFreshIndex handle);

// เพิ่มเวกเตอร์แบบ batch (vectors = [n * dim] float32 ติดกัน)
SPFreshStatus spfresh_add(SPFreshIndex handle,
                          const float* vectors,
                          size_t n,
                          const int64_t* ids);  // ส่ง nullptr ถ้าให้ระบบรันนิ่ง id เอง

// ค้นหา topk สำหรับ query 1 เวกเตอร์
SPFreshStatus spfresh_search(SPFreshIndex handle,
                             const float* query, // len=dim
                             int32_t topk,
                             int64_t* out_ids,   // len=topk (จำเป็น)
                             float* out_scores); // len=topk (optional; ส่ง nullptr ได้)

// persist/snapshot ลงดิสก์ (ถ้า API ภายในต้องเรียก)
SPFreshStatus spfresh_save(SPFreshIndex handle);

#ifdef __cplusplus
}
#endif
