# Hệ thống chữ viết tiếng Việt & Phương pháp gõ

Tài liệu toàn diện về hệ thống chữ viết tiếng Việt (chữ Quốc ngữ) và các phương pháp nhập liệu.

---

## Mục lục

1. [Tổng quan](#1-tổng-quan)
2. [Bảng chữ cái](#2-bảng-chữ-cái)
3. [Hệ thống nguyên âm](#3-hệ-thống-nguyên-âm)
4. [Hệ thống phụ âm](#4-hệ-thống-phụ-âm)
5. [Hệ thống thanh điệu](#5-hệ-thống-thanh-điệu)
6. [Cấu trúc âm tiết](#6-cấu-trúc-âm-tiết)
7. [Quy tắc đặt dấu thanh](#7-quy-tắc-đặt-dấu-thanh)
8. [Phương pháp gõ VNI](#8-phương-pháp-gõ-vni)
9. [Phương pháp gõ Telex](#9-phương-pháp-gõ-telex)
10. [Bảng mã Unicode](#10-bảng-mã-unicode)
11. [Tài liệu tham khảo](#11-tài-liệu-tham-khảo)

---

## 1. Tổng quan

### 1.1 Chữ Quốc ngữ

Chữ Quốc ngữ là hệ thống chữ viết chính thức của tiếng Việt, sử dụng bảng chữ cái Latin với các dấu phụ để biểu thị thanh điệu và các nguyên âm đặc biệt.

### 1.2 Đặc điểm

| Đặc điểm        | Giá trị            |
| --------------- | ------------------ |
| Loại chữ        | Alphabet (chữ cái) |
| Hướng viết      | Trái sang phải     |
| Số chữ cái      | 29                 |
| Số nguyên âm    | 12                 |
| Số phụ âm       | 17                 |
| Số thanh điệu   | 6                  |
| Số ký tự có dấu | 89 (bao gồm Đ/đ)   |

### 1.3 Lịch sử

- **Thế kỷ 17**: Các giáo sĩ Bồ Đào Nha và Pháp phát triển
- **1651**: Alexandre de Rhodes xuất bản từ điển Việt-Bồ-La
- **1945**: Trở thành chữ viết chính thức của Việt Nam

---

## 2. Bảng chữ cái

### 2.1 Bảng chữ cái tiếng Việt (29 chữ)

| STT | Chữ hoa | Chữ thường | Tên gọi |
| --- | ------- | ---------- | ------- |
| 1   | A       | a          | a       |
| 2   | Ă       | ă          | á       |
| 3   | Â       | â          | ớ       |
| 4   | B       | b          | bê      |
| 5   | C       | c          | xê      |
| 6   | D       | d          | dê      |
| 7   | Đ       | đ          | đê      |
| 8   | E       | e          | e       |
| 9   | Ê       | ê          | ê       |
| 10  | G       | g          | giê     |
| 11  | H       | h          | hát     |
| 12  | I       | i          | i       |
| 13  | K       | k          | ca      |
| 14  | L       | l          | e-lờ    |
| 15  | M       | m          | em-mờ   |
| 16  | N       | n          | en-nờ   |
| 17  | O       | o          | o       |
| 18  | Ô       | ô          | ô       |
| 19  | Ơ       | ơ          | ơ       |
| 20  | P       | p          | pê      |
| 21  | Q       | q          | cu/quy  |
| 22  | R       | r          | e-rờ    |
| 23  | S       | s          | ét-sì   |
| 24  | T       | t          | tê      |
| 25  | U       | u          | u       |
| 26  | Ư       | ư          | ư       |
| 27  | V       | v          | vê      |
| 28  | X       | x          | ích-xì  |
| 29  | Y       | y          | i dài   |

### 2.2 Các chữ không có trong tiếng Việt

Các chữ cái Latin sau **không có** trong bảng chữ cái tiếng Việt chính thức:

**F, J, W, Z**

> Tuy nhiên, chúng xuất hiện trong từ mượn và tên riêng nước ngoài.

---

## 3. Hệ thống nguyên âm

### 3.1 Nguyên âm đơn (12 nguyên âm)

#### 3.1.1 Phân loại theo vị trí lưỡi

| Vị trí        | Trước | Giữa | Sau |
| ------------- | ----- | ---- | --- |
| **Cao**       | i     | ư    | u   |
| **Giữa cao**  | ê     | ơ    | ô   |
| **Giữa thấp** | e     | â, ă | o   |
| **Thấp**      |       | a    |     |

#### 3.1.2 Phân loại theo độ mở miệng

| Độ mở          | Không tròn môi | Tròn môi |
| -------------- | -------------- | -------- |
| **Đóng (cao)** | i, ư           | u        |
| **Nửa đóng**   | ê, ơ           | ô        |
| **Nửa mở**     | e, â, ă        | o        |
| **Mở (thấp)**  | a              |          |

#### 3.1.3 Chi tiết từng nguyên âm

| Nguyên âm | IPA  | Mô tả               | Ví dụ           |
| --------- | ---- | ------------------- | --------------- |
| a         | /aː/ | Mở, giữa, dài       | ba, ca, ma      |
| ă         | /a/  | Mở, giữa, ngắn      | ăn, căn, măng   |
| â         | /ə/  | Nửa mở, giữa        | ân, cân, tân    |
| e         | /ɛ/  | Nửa mở, trước       | em, len, hen    |
| ê         | /e/  | Nửa đóng, trước     | ếch, bên, tên   |
| i         | /i/  | Đóng, trước         | in, kim, tim    |
| o         | /ɔ/  | Nửa mở, sau, tròn   | on, con, lon    |
| ô         | /o/  | Nửa đóng, sau, tròn | ông, công, bông |
| ơ         | /əː/ | Nửa đóng, giữa, dài | ơn, sơn, đơn    |
| u         | /u/  | Đóng, sau, tròn     | un, cun, bun    |
| ư         | /ɨ/  | Đóng, giữa          | ưng, dưng, hưng |
| y         | /i/  | Đóng, trước (như i) | y tá, ý kiến    |

### 3.2 Nguyên âm đôi (Diphthongs)

| Nguyên âm đôi | IPA   | Ví dụ              |
| ------------- | ----- | ------------------ |
| ai            | /aj/  | hai, mai, tai      |
| ao            | /aw/  | cao, sao, nao      |
| au            | /aw/  | sau, mau, dâu      |
| âu            | /əw/  | đâu, câu, sâu      |
| ay            | /aj/  | hay, may, say      |
| ây            | /əj/  | đây, cây, mây      |
| eo            | /ɛw/  | keo, theo, treo    |
| êu            | /ew/  | kêu, rêu, nêu      |
| ia            | /iə/  | kia, mía, lịa      |
| iê            | /iə/  | tiên, kiên, liên   |
| iu            | /iw/  | dịu, kíu, tiu      |
| oa            | /wa/  | hoa, toa, loa      |
| oă            | /wa/  | loắt, xoắn         |
| oe            | /wɛ/  | khoe, toe, xoe     |
| oi            | /ɔj/  | coi, đói, nói      |
| ôi            | /oj/  | tôi, hội, đối      |
| ơi            | /əːj/ | bơi, chơi, rơi     |
| ua            | /uə/  | mua, cua, lua      |
| uâ            | /uə/  | luân, tuân         |
| ưa            | /ɨə/  | mưa, cửa, lửa      |
| uê            | /uə/  | huệ, tuệ           |
| ui            | /uj/  | núi, dui, cúi      |
| ưi            | /ɨj/  | gửi, lửi           |
| uo            | /uə/  | thuở, muộn         |
| uô            | /uə/  | muốn, cuốn, luôn   |
| ươ            | /ɨə/  | mương, người, lười |
| uy            | /uj/  | huy, quy, tuy      |

### 3.3 Nguyên âm ba (Triphthongs)

| Nguyên âm ba | IPA   | Ví dụ             |
| ------------ | ----- | ----------------- |
| iêu          | /iəw/ | tiêu, kiều, liêu  |
| oai          | /waj/ | ngoài, loại, hoài |
| oay          | /waj/ | xoay, ngoáy       |
| oeo          | /wɛw/ | khoèo, ngoẹo      |
| uây          | /uəj/ | khuấy, quậy       |
| uôi          | /uəj/ | cuối, người, tuổi |
| ươi          | /ɨəj/ | mười, tươi, gửi   |
| ươu          | /ɨəw/ | rượu, hươu        |
| yêu          | /iəw/ | yêu, yếu          |
| uyê          | /uiə/ | khuyên, chuyên    |

---

## 4. Hệ thống phụ âm

### 4.1 Phụ âm đầu

#### 4.1.1 Phụ âm đơn (17 phụ âm)

| Phụ âm | IPA                 | Vị trí    | Cách phát âm            | Ví dụ         |
| ------ | ------------------- | --------- | ----------------------- | ------------- |
| b      | /ɓ/                 | Môi       | Tắc, hữu thanh, nội phá | ba, bé        |
| c      | /k/                 | Ngạc mềm  | Tắc, vô thanh           | ca, có        |
| d      | /z/ (Bắc) /j/ (Nam) | Chân răng | Xát, hữu thanh          | da, dê        |
| đ      | /ɗ/                 | Chân răng | Tắc, hữu thanh, nội phá | đi, đó        |
| g      | /ɣ/                 | Ngạc mềm  | Xát, hữu thanh          | gà, gỗ        |
| h      | /h/                 | Thanh hầu | Xát, vô thanh           | hoa, hát      |
| k      | /k/                 | Ngạc mềm  | Tắc, vô thanh           | kéo, kê       |
| l      | /l/                 | Chân răng | Bên                     | la, lá        |
| m      | /m/                 | Môi       | Mũi                     | ma, mẹ        |
| n      | /n/                 | Chân răng | Mũi                     | na, nói       |
| p      | /p/                 | Môi       | Tắc, vô thanh           | pin (từ mượn) |
| q      | /k/                 | Ngạc mềm  | Tắc (luôn đi với u)     | qua, quê      |
| r      | /z/ (Bắc) /r/ (Nam) | Chân răng | Xát/Rung                | ra, rồi       |
| s      | /s/                 | Chân răng | Xát, vô thanh           | sa, sao       |
| t      | /t/                 | Chân răng | Tắc, vô thanh           | ta, tôi       |
| v      | /v/                 | Môi-răng  | Xát, hữu thanh          | va, về        |
| x      | /s/                 | Chân răng | Xát, vô thanh           | xa, xanh      |

#### 4.1.2 Phụ âm đôi (Digraphs) - 11 phụ âm

| Phụ âm | IPA                 | Mô tả                   | Ví dụ      |
| ------ | ------------------- | ----------------------- | ---------- |
| ch     | /c/                 | Ngạc cứng, tắc          | cha, chị   |
| gh     | /ɣ/                 | Như g, trước e, ê, i    | ghe, ghi   |
| gi     | /z/ (Bắc) /j/ (Nam) | Như d                   | gia, giờ   |
| kh     | /x/                 | Ngạc mềm, xát           | khi, khô   |
| ng     | /ŋ/                 | Ngạc mềm, mũi           | nga, ngồi  |
| nh     | /ɲ/                 | Ngạc cứng, mũi          | nha, nhớ   |
| ph     | /f/                 | Môi-răng, xát           | pha, phố   |
| qu     | /kw/                | Ngạc mềm + môi          | qua, quê   |
| th     | /tʰ/                | Chân răng, tắc, bật hơi | tha, thì   |
| tr     | /c/ (Bắc) /ʈ/ (Nam) | Quặt lưỡi               | tra, trong |

#### 4.1.3 Phụ âm ba (Trigraph) - 1 phụ âm

| Phụ âm | IPA | Mô tả                 | Ví dụ      |
| ------ | --- | --------------------- | ---------- |
| ngh    | /ŋ/ | Như ng, trước e, ê, i | nghe, nghĩ |

### 4.2 Phụ âm cuối

Tiếng Việt chỉ cho phép một số phụ âm ở vị trí cuối âm tiết:

| Phụ âm cuối | IPA | Ví dụ      |
| ----------- | --- | ---------- |
| c           | /k/ | các, học   |
| ch          | /c/ | ách, ích   |
| m           | /m/ | cam, tâm   |
| n           | /n/ | can, tân   |
| ng          | /ŋ/ | cang, tang |
| nh          | /ɲ/ | anh, inh   |
| p           | /p/ | cập, tập   |
| t           | /t/ | cát, tất   |

### 4.3 Bảng phân bố phụ âm

| Vị trí →          | Môi   | Môi-răng | Chân răng | Quặt lưỡi | Ngạc cứng | Ngạc mềm | Thanh hầu |
| ----------------- | ----- | -------- | --------- | --------- | --------- | -------- | --------- |
| **Tắc vô thanh**  | p     |          | t         |           | ch        | c, k, q  |           |
| **Tắc hữu thanh** | b (ɓ) |          | đ (ɗ)     |           |           |          |           |
| **Mũi**           | m     |          | n         |           | nh        | ng       |           |
| **Xát vô thanh**  |       | ph (f)   | s, x      |           |           | kh       | h         |
| **Xát hữu thanh** |       | v        | d, gi, r  |           |           | g, gh    |           |
| **Bên**           |       |          | l         |           |           |          |           |

---

## 5. Hệ thống thanh điệu

### 5.1 Sáu thanh điệu

Tiếng Việt có 6 thanh điệu, tạo nên đặc trưng "tonal language":

| STT | Tên               | Ký hiệu | Số  | Contour   | Ví dụ | IPA     |
| --- | ----------------- | ------- | --- | --------- | ----- | ------- |
| 1   | Ngang (không dấu) | (không) | 1   | ˧ (33)    | ma    | /ma˧/   |
| 2   | Huyền             | `       | 2   | ˨˩ (21)   | mà    | /ma˨˩/  |
| 3   | Sắc               | ´       | 3   | ˧˥ (35)   | má    | /ma˧˥/  |
| 4   | Hỏi               | ̉        | 4   | ˧˩˧ (313) | mả    | /ma˧˩˧/ |
| 5   | Ngã               | ~       | 5   | ˧˥ˀ (35ˀ) | mã    | /ma˧˥ˀ/ |
| 6   | Nặng              | ̣        | 6   | ˧˩ˀ (21ˀ) | mạ    | /ma˧˩ˀ/ |

### 5.2 Đặc điểm thanh điệu

| Thanh | Cao độ | Đường nét      | Đặc biệt         |
| ----- | ------ | -------------- | ---------------- |
| Ngang | Trung  | Bằng phẳng     | Không dấu        |
| Huyền | Thấp   | Đi xuống       |                  |
| Sắc   | Cao    | Đi lên         |                  |
| Hỏi   | Thấp   | Xuống rồi lên  | Giọng lượn       |
| Ngã   | Cao    | Lên rồi nghẽn  | Có thanh hầu hóa |
| Nặng  | Thấp   | Xuống và nghẽn | Có thanh hầu hóa |

### 5.3 Biểu đồ thanh điệu

```
Cao   5 ─────────────────────────
      4 ─────────────────────────
Trung 3 ────●━━━━━━━━━━━━━━━━━━━━  Ngang (33)
      2 ─────────────────────────
Thấp  1 ─────────────────────────

Cao   5 ──────────────────●━━━━━━  Sắc (35)
      4 ─────────────────/───────
Trung 3 ────●━━━━━━━━━━━/━━━━━━━━
      2 ─────────────────────────
Thấp  1 ─────────────────────────

Cao   5 ─────────────────────────
      4 ─────────────────────────
Trung 3 ────●━━━━━━━━━━━━━━━━━━━━
      2 ────────\────────────────  Huyền (21)
Thấp  1 ─────────●━━━━━━━━━━━━━━━

Cao   5 ─────────────────────────
      4 ─────────────────────────
Trung 3 ────●━━━━━━━━━━━━━●━━━━━━  Hỏi (313)
      2 ────────\────────/───────
Thấp  1 ─────────●━━━━━━━━━━━━━━━

Cao   5 ──────────────────●⁎━━━━━  Ngã (35ˀ) ⁎=nghẽn
      4 ─────────────────/───────
Trung 3 ────●━━━━━━━━━━━/━━━━━━━━
      2 ─────────────────────────
Thấp  1 ─────────────────────────

Cao   5 ─────────────────────────
      4 ─────────────────────────
Trung 3 ────●━━━━━━━━━━━━━━━━━━━━
      2 ────────\────────────────  Nặng (21ˀ) ⁎=nghẽn
Thấp  1 ─────────●⁎━━━━━━━━━━━━━━
```

### 5.4 Phương ngữ và thanh điệu

| Vùng     | Số thanh | Ghi chú                |
| -------- | -------- | ---------------------- |
| Bắc Bộ   | 6        | Đầy đủ 6 thanh         |
| Trung Bộ | 5        | Hỏi và Ngã thường nhập |
| Nam Bộ   | 5        | Hỏi và Ngã thường nhập |

---

## 6. Cấu trúc âm tiết

### 6.1 Công thức tổng quát

```
Âm tiết = (C₁)(G)V(C₂) + T
```

Hoặc chi tiết hơn:

```
(Phụ âm đầu)(Âm đệm)(Nguyên âm chính)(Âm cuối) + Thanh điệu
```

### 6.2 Các thành phần

| Ký hiệu | Thành phần      | Bắt buộc | Giá trị có thể                                                                             |
| ------- | --------------- | -------- | ------------------------------------------------------------------------------------------ |
| C₁      | Phụ âm đầu      | Không    | b, c, ch, d, đ, g, gh, gi, h, k, kh, l, m, n, ng, ngh, nh, p, ph, q, r, s, t, th, tr, v, x |
| G       | Âm đệm          | Không    | o, u                                                                                       |
| V       | Nguyên âm chính | **Có**   | a, ă, â, e, ê, i, o, ô, ơ, u, ư, y + các nguyên âm đôi/ba                                  |
| C₂      | Âm cuối         | Không    | c, ch, m, n, ng, nh, p, t + i/y, o/u (bán nguyên âm)                                       |
| T       | Thanh điệu      | **Có**   | ngang, huyền, sắc, hỏi, ngã, nặng                                                          |

### 6.3 Ví dụ phân tích

| Từ     | C₁  | G   | V   | C₂  | T     | Ghi chú               |
| ------ | --- | --- | --- | --- | ----- | --------------------- |
| a      | -   | -   | a   | -   | ngang | Chỉ có nguyên âm      |
| an     | -   | -   | a   | n   | ngang | Nguyên âm + âm cuối   |
| ba     | b   | -   | a   | -   | ngang | Phụ âm + nguyên âm    |
| ban    | b   | -   | a   | n   | ngang | Đầy đủ trừ âm đệm     |
| hoa    | h   | o   | a   | -   | ngang | Có âm đệm             |
| hoàn   | h   | o   | a   | n   | huyền | Đầy đủ các thành phần |
| toán   | t   | o   | a   | n   | sắc   | Đầy đủ các thành phần |
| qua    | qu  | -   | a   | -   | ngang | qu là một đơn vị      |
| quán   | qu  | -   | a   | n   | sắc   |                       |
| người  | ng  | -   | ươ  | i   | huyền | Nguyên âm đôi         |
| trường | tr  | -   | ươ  | ng  | huyền | Nguyên âm đôi         |
| khuya  | kh  | u   | y   | a   | ngang |                       |
| nguyễn | ng  | u   | yê  | n   | ngã   | Nguyên âm ba          |

### 6.4 Quy tắc kết hợp

#### 6.4.1 Âm đệm

- **o** đứng trước: a, ă, e (hoa, hoặc, hoe)
- **u** đứng trước: a, â, ê, y, yê (qua, quân, quê, quy, khuyên)

#### 6.4.2 Phụ âm cuối

| Nhóm | Phụ âm       | Kết hợp với             |
| ---- | ------------ | ----------------------- |
| Mũi  | m, n, ng, nh | Tất cả thanh điệu       |
| Tắc  | p, t, c, ch  | Chỉ thanh sắc hoặc nặng |

> **Quy tắc**: Âm tiết kết thúc bằng p, t, c, ch chỉ mang thanh sắc hoặc nặng.
> Ví dụ: cấp/cập (✓), cảp/cãp/càp (✗)

---

## 7. Quy tắc đặt dấu thanh

### 7.1 Nguyên tắc cơ bản

Dấu thanh đặt trên **nguyên âm chính** (âm chính) của âm tiết.

### 7.2 Quy tắc chi tiết

#### 7.2.1 Một nguyên âm

Dấu đặt trực tiếp trên nguyên âm đó:

```
bá, bà, bả, bã, bạ
```

#### 7.2.2 Hai nguyên âm - Âm đóng (có phụ âm cuối)

Dấu đặt trên nguyên âm **thứ hai**:

| Ví dụ | Phân tích  |
| ----- | ---------- |
| toán  | to-**á**-n |
| hoàn  | ho-**à**-n |
| tiến  | ti-**ế**-n |
| muốn  | mu-**ố**-n |
| biển  | bi-**ể**-n |

#### 7.2.3 Hai nguyên âm - Âm mở (không có phụ âm cuối)

**Nhóm 1: Âm đệm + Âm chính** → Dấu trên nguyên âm **thứ hai**

| Pattern    | Ví dụ                   | Dấu trên |
| ---------- | ----------------------- | -------- |
| oa         | hoà, hoá, hoả, hoã, hoạ | a        |
| oe         | khoẻ, hoè               | e        |
| uy         | quý, quỳ, quỷ, quỹ, quỵ | y        |
| uê         | huế                     | ê        |
| ua (sau q) | quá, quà                | a        |

**Nhóm 2: Âm chính + Bán nguyên âm** → Dấu trên nguyên âm **thứ nhất**

| Pattern | Ví dụ                   | Dấu trên |
| ------- | ----------------------- | -------- |
| ai      | mái, mài, mải, mãi, mại | a        |
| ao      | cáo, cào, cảo, cão, cạo | a        |
| au      | sáu, sàu, sảu, sãu, sạu | a        |
| ay      | máy, mày, mảy, mãy, mạy | a        |
| âu      | đấu, đầu, đẩu, đẫu, đậu | â        |
| ây      | đấy, đầy, đẩy, đẫy, đậy | â        |
| eo      | kéo, kèo, kẻo, kẽo, kẹo | e        |
| êu      | kêu, kều, kểu, kễu, kệu | ê        |
| iu      | díu, dìu, dỉu, dĩu, dịu | i        |
| oi      | đói, đòi, đỏi, đõi, đọi | o        |
| ôi      | tối, tồi, tổi, tỗi, tội | ô        |
| ơi      | bơi, bời, bởi, bỡi, bợi | ơ        |
| ui      | túi, tùi, tủi, tũi, tụi | u        |
| ưi      | gửi, gừi                | ư        |

#### 7.2.4 Ba nguyên âm

Dấu đặt trên nguyên âm **giữa** (thường là nguyên âm chính):

| Pattern | Ví dụ                                  | Dấu trên |
| ------- | -------------------------------------- | -------- |
| iêu     | tiếu, tiều, tiểu, tiễu, tiệu           | ê        |
| yêu     | yếu, yều, yểu, yễu, yệu                | ê        |
| ươi     | mười, mười, mưởi, mưỡi, mượi           | ơ        |
| ươu     | rượu, rườu                             | ơ        |
| oai     | ngoái, ngoài, ngoải, ngoãi, ngoại      | a        |
| oay     | xoáy, xoày, xoảy, xoãy, xoạy           | a        |
| uôi     | cuối, cuồi, cuổi, cuỗi, cuội           | ô        |
| uyê     | khuyến, khuyền, khuyển, khuyễn, khuyện | ê        |
| uây     | khuấy, khuầy                           | â        |

#### 7.2.5 Nguyên âm có dấu phụ (ă, â, ê, ô, ơ, ư)

Nếu có nguyên âm đã có dấu phụ, dấu thanh **ưu tiên** đặt trên nguyên âm đó:

| Ví dụ | Giải thích                |
| ----- | ------------------------- |
| sứa   | ư có dấu phụ → dấu trên ư |
| đời   | ơ có dấu phụ → dấu trên ơ |
| luật  | â có dấu phụ → dấu trên â |

### 7.3 Tóm tắt quy tắc

```
┌─────────────────────────────────────────────────────────┐
│                  QUY TẮC ĐẶT DẤU THANH                  │
├─────────────────────────────────────────────────────────┤
│ 1. Một nguyên âm → Đặt trên nguyên âm đó               │
│                                                         │
│ 2. Hai nguyên âm:                                       │
│    - Có phụ âm cuối → Đặt trên nguyên âm thứ 2         │
│    - Âm đệm + Âm chính (oa,oe,uy) → Nguyên âm thứ 2    │
│    - Âm chính + Bán nguyên âm (ai,ao,au...) → Thứ 1   │
│                                                         │
│ 3. Ba nguyên âm → Đặt trên nguyên âm giữa              │
│                                                         │
│ 4. Có nguyên âm dấu phụ (ă,â,ê,ô,ơ,ư) → Ưu tiên       │
└─────────────────────────────────────────────────────────┘
```

---

## 8. Phương pháp gõ VNI

### 8.1 Tổng quan

VNI sử dụng các phím số 0-9 để thêm dấu vào chữ cái.

### 8.2 Bảng phím VNI

| Phím  | Chức năng     | Áp dụng                 | Kết quả                 |
| ----- | ------------- | ----------------------- | ----------------------- |
| **1** | Dấu sắc       | a,ă,â,e,ê,i,o,ô,ơ,u,ư,y | á,ắ,ấ,é,ế,í,ó,ố,ớ,ú,ứ,ý |
| **2** | Dấu huyền     | a,ă,â,e,ê,i,o,ô,ơ,u,ư,y | à,ằ,ầ,è,ề,ì,ò,ồ,ờ,ù,ừ,ỳ |
| **3** | Dấu hỏi       | a,ă,â,e,ê,i,o,ô,ơ,u,ư,y | ả,ẳ,ẩ,ẻ,ể,ỉ,ỏ,ổ,ở,ủ,ử,ỷ |
| **4** | Dấu ngã       | a,ă,â,e,ê,i,o,ô,ơ,u,ư,y | ã,ẵ,ẫ,ẽ,ễ,ĩ,õ,ỗ,ỡ,ũ,ữ,ỹ |
| **5** | Dấu nặng      | a,ă,â,e,ê,i,o,ô,ơ,u,ư,y | ạ,ặ,ậ,ẹ,ệ,ị,ọ,ộ,ợ,ụ,ự,ỵ |
| **6** | Dấu mũ (^)    | a,e,o                   | â,ê,ô                   |
| **7** | Dấu móc       | o,u                     | ơ,ư                     |
| **8** | Dấu trăng (˘) | a                       | ă                       |
| **9** | Gạch ngang    | d                       | đ                       |
| **0** | Xóa dấu       | Tất cả                  | Xóa dấu cuối            |

### 8.3 Ví dụ VNI

| Input  | Output | Giải thích |
| ------ | ------ | ---------- |
| a1     | á      | a + sắc    |
| a2     | à      | a + huyền  |
| a3     | ả      | a + hỏi    |
| a4     | ã      | a + ngã    |
| a5     | ạ      | a + nặng   |
| a6     | â      | a + mũ     |
| a61    | ấ      | â + sắc    |
| a8     | ă      | a + trăng  |
| a81    | ắ      | ă + sắc    |
| o7     | ơ      | o + móc    |
| o72    | ờ      | ơ + huyền  |
| u7     | ư      | u + móc    |
| u72    | ừ      | ư + huyền  |
| d9     | đ      | d + gạch   |
| Vie65t | Việt   | V+i+ệ+t    |
| tu72   | từ     | t+ừ        |

### 8.4 Hoàn tác VNI

Nhấn phím hai lần để hoàn tác:

| Input | Output | Giải thích     |
| ----- | ------ | -------------- |
| a11   | a1     | Hoàn tác sắc   |
| a66   | a6     | Hoàn tác mũ    |
| o77   | o7     | Hoàn tác móc   |
| a88   | a8     | Hoàn tác trăng |

---

## 9. Phương pháp gõ Telex

### 9.1 Tổng quan

Telex sử dụng các phím chữ cái để thêm dấu, tận dụng các chữ không có trong tiếng Việt (f, j, w, z).

### 9.2 Bảng phím Telex

#### 9.2.1 Dấu thanh

| Phím  | Dấu     | Áp dụng          | Kết quả                 |
| ----- | ------- | ---------------- | ----------------------- |
| **s** | Sắc     | Tất cả nguyên âm | á,ắ,ấ,é,ế,í,ó,ố,ớ,ú,ứ,ý |
| **f** | Huyền   | Tất cả nguyên âm | à,ằ,ầ,è,ề,ì,ò,ồ,ờ,ù,ừ,ỳ |
| **r** | Hỏi     | Tất cả nguyên âm | ả,ẳ,ẩ,ẻ,ể,ỉ,ỏ,ổ,ở,ủ,ử,ỷ |
| **x** | Ngã     | Tất cả nguyên âm | ã,ẵ,ẫ,ẽ,ễ,ĩ,õ,ỗ,ỡ,ũ,ữ,ỹ |
| **j** | Nặng    | Tất cả nguyên âm | ạ,ặ,ậ,ẹ,ệ,ị,ọ,ộ,ợ,ụ,ự,ỵ |
| **z** | Xóa dấu | Tất cả           | Xóa dấu                 |

#### 9.2.2 Dấu phụ

| Phím   | Dấu       | Áp dụng | Kết quả |
| ------ | --------- | ------- | ------- |
| **aa** | Mũ (^)    | a       | â       |
| **ee** | Mũ (^)    | e       | ê       |
| **oo** | Mũ (^)    | o       | ô       |
| **aw** | Trăng (˘) | a       | ă       |
| **ow** | Móc       | o       | ơ       |
| **uw** | Móc       | u       | ư       |
| **dd** | Gạch      | d       | đ       |

### 9.3 Ví dụ Telex

| Input  | Output | Giải thích |
| ------ | ------ | ---------- |
| as     | á      | a + sắc    |
| af     | à      | a + huyền  |
| ar     | ả      | a + hỏi    |
| ax     | ã      | a + ngã    |
| aj     | ạ      | a + nặng   |
| aa     | â      | a + mũ     |
| aas    | ấ      | â + sắc    |
| aw     | ă      | a + trăng  |
| aws    | ắ      | ă + sắc    |
| ow     | ơ      | o + móc    |
| owf    | ờ      | ơ + huyền  |
| uw     | ư      | u + móc    |
| uwf    | ừ      | ư + huyền  |
| dd     | đ      | d + gạch   |
| Vieetj | Việt   | V+iệ+t     |
| tuwf   | từ     | t+ừ        |

### 9.4 Hoàn tác Telex

Nhấn phím hai lần để hoàn tác:

| Input | Output | Giải thích     |
| ----- | ------ | -------------- |
| ass   | as     | Hoàn tác sắc   |
| aaa   | aa     | Hoàn tác mũ    |
| aww   | aw     | Hoàn tác trăng |
| oww   | ow     | Hoàn tác móc   |

### 9.5 Thứ tự linh hoạt

Telex cho phép gõ dấu phụ và dấu thanh theo thứ tự bất kỳ:

| Tương đương   |
| ------------- |
| owf = ofw = ờ |
| aas = asa = ấ |
| uws = usw = ứ |

---

## 10. Bảng mã Unicode

### 10.1 Nguyên âm có dấu

#### 10.1.1 Chữ thường

| Gốc      | Sắc      | Huyền    | Hỏi      | Ngã      | Nặng     |
| -------- | -------- | -------- | -------- | -------- | -------- |
| a (0061) | á (00E1) | à (00E0) | ả (1EA3) | ã (00E3) | ạ (1EA1) |
| ă (0103) | ắ (1EAF) | ằ (1EB1) | ẳ (1EB3) | ẵ (1EB5) | ặ (1EB7) |
| â (00E2) | ấ (1EA5) | ầ (1EA7) | ẩ (1EA9) | ẫ (1EAB) | ậ (1EAD) |
| e (0065) | é (00E9) | è (00E8) | ẻ (1EBB) | ẽ (1EBD) | ẹ (1EB9) |
| ê (00EA) | ế (1EBF) | ề (1EC1) | ể (1EC3) | ễ (1EC5) | ệ (1EC7) |
| i (0069) | í (00ED) | ì (00EC) | ỉ (1EC9) | ĩ (0129) | ị (1ECB) |
| o (006F) | ó (00F3) | ò (00F2) | ỏ (1ECF) | õ (00F5) | ọ (1ECD) |
| ô (00F4) | ố (1ED1) | ồ (1ED3) | ổ (1ED5) | ỗ (1ED7) | ộ (1ED9) |
| ơ (01A1) | ớ (1EDB) | ờ (1EDD) | ở (1EDF) | ỡ (1EE1) | ợ (1EE3) |
| u (0075) | ú (00FA) | ù (00F9) | ủ (1EE7) | ũ (0169) | ụ (1EE5) |
| ư (01B0) | ứ (1EE9) | ừ (1EEB) | ử (1EED) | ữ (1EEF) | ự (1EF1) |
| y (0079) | ý (00FD) | ỳ (1EF3) | ỷ (1EF7) | ỹ (1EF9) | ỵ (1EF5) |

#### 10.1.2 Chữ hoa

| Gốc      | Sắc      | Huyền    | Hỏi      | Ngã      | Nặng     |
| -------- | -------- | -------- | -------- | -------- | -------- |
| A (0041) | Á (00C1) | À (00C0) | Ả (1EA2) | Ã (00C3) | Ạ (1EA0) |
| Ă (0102) | Ắ (1EAE) | Ằ (1EB0) | Ẳ (1EB2) | Ẵ (1EB4) | Ặ (1EB6) |
| Â (00C2) | Ấ (1EA4) | Ầ (1EA6) | Ẩ (1EA8) | Ẫ (1EAA) | Ậ (1EAC) |
| E (0045) | É (00C9) | È (00C8) | Ẻ (1EBA) | Ẽ (1EBC) | Ẹ (1EB8) |
| Ê (00CA) | Ế (1EBE) | Ề (1EC0) | Ể (1EC2) | Ễ (1EC4) | Ệ (1EC6) |
| I (0049) | Í (00CD) | Ì (00CC) | Ỉ (1EC8) | Ĩ (0128) | Ị (1ECA) |
| O (004F) | Ó (00D3) | Ò (00D2) | Ỏ (1ECE) | Õ (00D5) | Ọ (1ECC) |
| Ô (00D4) | Ố (1ED0) | Ồ (1ED2) | Ổ (1ED4) | Ỗ (1ED6) | Ộ (1ED8) |
| Ơ (01A0) | Ớ (1EDA) | Ờ (1EDC) | Ở (1EDE) | Ỡ (1EE0) | Ợ (1EE2) |
| U (0055) | Ú (00DA) | Ù (00D9) | Ủ (1EE6) | Ũ (0168) | Ụ (1EE4) |
| Ư (01AF) | Ứ (1EE8) | Ừ (1EEA) | Ử (1EEC) | Ữ (1EEE) | Ự (1EF0) |
| Y (0059) | Ý (00DD) | Ỳ (1EF2) | Ỷ (1EF6) | Ỹ (1EF8) | Ỵ (1EF4) |

### 10.2 Chữ Đ

| Dạng       | Ký tự | Unicode |
| ---------- | ----- | ------- |
| Chữ hoa    | Đ     | U+0110  |
| Chữ thường | đ     | U+0111  |

### 10.3 Thống kê

| Loại                    | Số lượng    |
| ----------------------- | ----------- |
| Nguyên âm thường có dấu | 12 × 6 = 72 |
| Nguyên âm hoa có dấu    | 12 × 6 = 72 |
| Đ/đ                     | 2           |
| **Tổng ký tự đặc biệt** | **146**     |

---

## 11. Tài liệu tham khảo

### 11.1 Wikipedia

- [Vietnamese alphabet](https://en.wikipedia.org/wiki/Vietnamese_alphabet) - Bảng chữ cái tiếng Việt
- [Vietnamese phonology](https://en.wikipedia.org/wiki/Vietnamese_phonology) - Ngữ âm học tiếng Việt
- [Vietnamese language](https://en.wikipedia.org/wiki/Vietnamese_language) - Ngôn ngữ tiếng Việt
- [Vietnamese tones](https://en.wikipedia.org/wiki/Vietnamese_language#Tones) - Thanh điệu tiếng Việt
- [Chữ Quốc ngữ](https://vi.wikipedia.org/wiki/Ch%E1%BB%AF_Qu%E1%BB%91c_ng%E1%BB%AF) - Chữ Quốc ngữ (tiếng Việt)
- [VNI](https://en.wikipedia.org/wiki/VNI) - Phương pháp gõ VNI
- [Telex (input method)](<https://en.wikipedia.org/wiki/Telex_(input_method)>) - Phương pháp gõ Telex
- [Vietnamese language and computers](https://en.wikipedia.org/wiki/Vietnamese_language_and_computers) - Tiếng Việt và máy tính
- [Quy tắc đặt dấu thanh](https://vi.wikipedia.org/wiki/Quy_tắc_đặt_dấu_thanh_của_chữ_Quốc_ngữ) - Quy tắc đặt dấu

### 11.2 Công cụ và phần mềm

- [UniKey](https://www.unikey.org/) - Bộ gõ tiếng Việt phổ biến nhất
- [OpenKey](https://open-key.org/) - Bộ gõ tiếng Việt mã nguồn mở
- [EVKey](https://evkey.vn/) - Bộ gõ tiếng Việt

### 11.3 Tài liệu kỹ thuật

- [VietUnicode Input Methods](https://vietunicode.sourceforge.net/inputmethod.html) - Chi tiết các phương pháp nhập
- [Microsoft Vietnamese IME](https://learn.microsoft.com/en-us/globalization/input/vietnamese-ime) - Tài liệu Microsoft
- [Unicode Vietnamese](https://www.unicode.org/charts/PDF/U1E00.pdf) - Bảng mã Unicode Latin Extended Additional

### 11.4 Tiêu chuẩn Việt Nam

- **TCVN 5712:1993** - Công nghệ thông tin - Bộ mã tiêu chuẩn tiếng Việt 8 bit
- **TCVN 6909:2001** - Công nghệ thông tin - Bộ mã ký tự tiếng Việt 16 bit
- **TCVN 5773:1993** - Bàn phím máy tính - Bố trí phím cho tiếng Việt

### 11.5 Sách tham khảo

- Nguyễn Đình Hoà (1997). _Vietnamese: Tiếng Việt không son phấn_. John Benjamins Publishing.
- Thompson, Laurence C. (1965). _A Vietnamese Grammar_. University of Washington Press.
- Đoàn Thiện Thuật (1977). _Ngữ âm tiếng Việt_. NXB Đại học và Trung học chuyên nghiệp.

---

## Changelog

- **2025-12-07**: Tạo tài liệu toàn diện
  - Hệ thống hóa bảng chữ cái tiếng Việt
  - Chi tiết hệ thống nguyên âm (đơn, đôi, ba)
  - Chi tiết hệ thống phụ âm (đầu, cuối)
  - Hệ thống 6 thanh điệu với biểu đồ
  - Cấu trúc âm tiết tiếng Việt
  - Quy tắc đặt dấu thanh chi tiết
  - Phương pháp gõ VNI (sửa 7=móc, 8=trăng)
  - Phương pháp gõ Telex
  - Bảng mã Unicode đầy đủ
  - Tài liệu tham khảo toàn diện
