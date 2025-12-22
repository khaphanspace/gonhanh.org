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
| **Giữông a thấp** | e     | â, ă | o   |
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

#### 3.2.1 Phân loại theo hướng phát âm

**A. Nguyên âm đôi hướng lên (Rising Diphthongs)**

Nguyên âm đầu là âm đệm/glide, nguyên âm sau là âm chính.

| Pattern | IPA  | Loại | Ví dụ | Ghi chú |
|---------|------|------|-------|---------|
| ia      | /iə/ | i → a | kia, mía, lía | Âm mở |
| iê      | /iə/ | i → ê | tiên, kiên, liên | Compound vowel |
| ua      | /uə/ | u → a | mua, cua, lua | Âm mở |
| uô      | /uə/ | u → ô | muốn, cuốn, luôn | Compound vowel |
| ưa      | /ɨə/ | ư → a | mưa, cửa, lửa | Âm mở |
| ươ      | /ɨə/ | ư → ơ | mương, người, được | Compound vowel |

**B. Nguyên âm đôi hướng xuống - Kết thúc -i/-y (Falling to -i/-y)**

Nguyên âm đầu là âm chính, nguyên âm sau là bán nguyên âm cuối.

| Pattern | IPA   | Ví dụ | Ghi chú |
|---------|-------|-------|---------|
| ai      | /aj/  | hai, mai, tai | |
| ay      | /aj/  | hay, may, say | Khác "ai" về chính tả |
| ây      | /əj/  | đây, cây, mây | |
| oi      | /ɔj/  | coi, đói, nói | |
| ôi      | /oj/  | tôi, hội, đối | |
| ơi      | /əːj/ | bơi, chơi, rơi | |
| ui      | /uj/  | núi, dui, cúi | |
| ưi      | /ɨj/  | gửi, lửi | Ít từ |
| iu      | /iw/  | dịu, kíu, tiu | |

**C. Nguyên âm đôi hướng xuống - Kết thúc -u/-o (Falling to -u/-o)**

| Pattern | IPA  | Ví dụ | Ghi chú |
|---------|------|-------|---------|
| ao      | /aw/ | cao, sao, nao | |
| au      | /aw/ | sau, mau, dâu | Khác "ao" về chính tả |
| âu      | /əw/ | đâu, câu, sâu | |
| êu      | /ew/ | kêu, rêu, nêu | |
| eo      | /ɛw/ | keo, theo, treo | |
| **ưu**  | /ɨw/ | **lưu, hưu, ngưu, cứu** | **Quan trọng!** |

**D. Tổ hợp âm đệm + âm chính (Medial + Main)**

Nguyên âm đầu đóng vai trò âm đệm (không phải âm chính).

| Pattern | IPA  | Âm đệm | Âm chính | Ví dụ | Điều kiện |
|---------|------|--------|----------|-------|-----------|
| oa      | /wa/ | o      | a        | hoa, toa, loa | Sau phụ âm |
| oă      | /wa/ | o      | ă        | xoắn, loắt, hoặc | Sau phụ âm |
| oe      | /wɛ/ | o      | e        | khoe, xoe, toe | Sau phụ âm |
| uê      | /wə/ | u      | ê        | huệ, tuệ | |
| uy      | /uj/ | u      | y        | quy, huy, tuy | |
| uâ      | /wə/ | u      | â        | luân, tuân, quân | Hiếm |

> **Lưu ý quan trọng về "ua"**:
> - Sau "q" (qua, quả): u là âm đệm, a là âm chính → dấu trên a
> - Âm tiết mở không có "q" (mua, cua): u là âm chính, a là bán nguyên âm → dấu trên u
> - Âm tiết đóng có phụ âm cuối (chuẩn, thuận): u là âm đệm, â là âm chính → dấu trên â

#### 3.2.2 Nguyên âm đôi đặc biệt (Special Cases)

| Pattern | IPA  | Ví dụ | Ghi chú |
|---------|------|-------|---------|
| uo      | /uə/ | thuở | Rất hiếm, chỉ vài từ |

#### 3.2.3 Quy tắc kết hợp nguyên âm

> **Xem chi tiết**: [Section 7.6 - Ma trận tổng hợp cho IME](#76-ma-trận-tổng-hợp-cho-ime-consolidated-ime-matrix) chứa danh sách đầy đủ 39 patterns hợp lệ.

### 3.3 Nguyên âm ba (Triphthongs)

#### 3.3.1 Danh sách đầy đủ

| Nguyên âm ba | IPA   | Cấu trúc | Ví dụ |
| ------------ | ----- | -------- | ----- |
| iêu          | /iəw/ | i + ê + u | tiêu, kiều, liêu |
| yêu          | /iəw/ | y + ê + u | yêu, yếu |
| oai          | /waj/ | o + a + i | ngoài, loại, hoài |
| oay          | /waj/ | o + a + y | xoay, ngoáy |
| oeo          | /wɛw/ | o + e + o | khoèo, ngoẹo |
| uây          | /uəj/ | u + â + y | khuấy, quậy |
| uôi          | /uəj/ | u + ô + i | cuối, tuổi, buổi |
| ươi          | /ɨəj/ | ư + ơ + i | mười, tươi, lười |
| ươu          | /ɨəw/ | ư + ơ + u | rượu, hươu, bướu |
| uyê          | /uiə/ | u + y + ê | khuyên, chuyên, quyền |

#### 3.3.2 Quy tắc đặt dấu thanh cho nguyên âm ba

| Pattern | Dấu thanh trên | Lý do |
|---------|---------------|-------|
| iêu, yêu | ê (giữa) | ê có dấu phụ |
| oai, oay | a (giữa) | a là âm chính |
| oeo | e (giữa) | e là âm chính |
| uây | â (giữa) | â có dấu phụ |
| uôi | ô (giữa) | ô có dấu phụ |
| ươi, ươu | ơ (giữa) | ơ có dấu phụ |
| uyê | ê (cuối) | ê có dấu phụ |

#### 3.3.3 Các tổ hợp ba nguyên âm KHÔNG tồn tại

| Pattern | Lý do |
|---------|-------|
| aiu, aui, eau | Không theo cấu trúc tiếng Việt |
| oui, uoi (không dấu) | Phải là uôi |
| ieo, eoi | Không tồn tại |

### 3.4 Tổng hợp quy tắc nguyên âm cho bộ gõ

#### 3.4.1 Bảng tra cứu nhanh - Nguyên âm hợp lệ

```
NGUYÊN ÂM ĐƠN (12):
a, ă, â, e, ê, i, o, ô, ơ, u, ư, y

NGUYÊN ÂM ĐÔI HỢP LỆ (27):
├── Hướng lên: ia, iê, ua, uô, ưa, ươ
├── Hướng xuống (-i/-y): ai, ay, ây, oi, ôi, ơi, ui, ưi, iu
├── Hướng xuống (-u/-o): ao, au, âu, êu, eo, ưu
├── Âm đệm: oa, oă, oe, uâ, uê, uy
└── Đặc biệt: uo (hiếm)

NGUYÊN ÂM BA HỢP LỆ (10):
iêu, yêu, oai, oay, oeo, uây, uôi, ươi, ươu, uyê

KHÔNG HỢP LỆ:
ou, yo, eu, ae, yi, wu, aiu, eau...
```

#### 3.4.2 Ma trận kết hợp nguyên âm

> **Xem chi tiết**: [Section 7.6 - Ma trận tổng hợp cho IME](#76-ma-trận-tổng-hợp-cho-ime-consolidated-ime-matrix) - bảng đầy đủ 39 patterns với Base Keys, Tone position, Modifier rules.

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

> **Lưu ý**: Trong tên riêng từ ngôn ngữ dân tộc thiểu số (Ê Đê, Ba Na, Gia Rai...), có thể xuất hiện:
> - Chữ **k** ở cuối âm tiết: Đắk Lắk, Đắk Nông, Búk
> - Phụ âm đầu **kr**: Krông Búk, Krông Ana
>
> Bộ gõ hỗ trợ các trường hợp này.

### 4.3 Bảng phân bố phụ âm

| Vị trí →          | Môi   | Môi-răng | Chân răng | Quặt lưỡi | Ngạc cứng | Ngạc mềm | Thanh hầu |
| ----------------- | ----- | -------- | --------- | --------- | --------- | -------- | --------- |
| **Tắc vô thanh**  | p     |          | t         |           | ch        | c, k, q  |           |
| **Tắc hữu thanh** | b (ɓ) |          | đ (ɗ)     |           |           |          |           |
| **Mũi**           | m     |          | n         |           | nh        | ng       |           |
| **Xát vô thanh**  |       | ph (f)   | s, x      |           |           | kh       | h         |
| **Xát hữu thanh** |       | v        | d, gi, r  |           |           | g, gh    |           |
| **Bên**           |       |          | l         |           |           |          |           |

### 4.4 Quy tắc Chính tả Phụ âm (Orthographic Rules)

> **Mục đích**: Các quy tắc viết chính tả bắt buộc - quan trọng cho việc validate và xử lý trong bộ gõ.

#### 4.4.1 Quy tắc C / K / Q

Ba chữ cái **c**, **k**, **q** đều biểu thị âm vị /k/ nhưng phân bố khác nhau:

| Chữ cái | Đứng trước nguyên âm | Ví dụ |
| ------- | -------------------- | ----- |
| **c**   | a, ă, â, o, ô, ơ, u, ư | ca, căn, cân, co, cô, cơ, cu, cư |
| **k**   | e, ê, i, y           | ke, kê, ki, ky |
| **q**   | luôn đi với **u** thành **qu** | qua, quê, qui, quy |

```
QUY TẮC:
├── C trước nguyên âm hàng sau: a, ă, â, o, ô, ơ, u, ư
├── K trước nguyên âm hàng trước: e, ê, i, y
└── Q + U = QU (là một đơn vị phụ âm)

VÍ DỤ:
├── ✓ ca, cô, cu, căn, cơm
├── ✓ kẻ, kê, ki, ký
├── ✓ qua, quê, quy, quả
├── ✗ ce, ci (phải viết: ke, ki)
└── ✗ ka, ko (phải viết: ca, co)
```

#### 4.4.2 Quy tắc G / GH

Hai cách viết **g** và **gh** đều biểu thị âm vị /ɣ/:

| Chữ cái | Đứng trước nguyên âm | Ví dụ |
| ------- | -------------------- | ----- |
| **g**   | a, ă, â, o, ô, ơ, u, ư | ga, găng, gân, go, gô, gơ, gu, gư |
| **gh**  | e, ê, i              | ghe, ghế, ghi |

```
QUY TẮC:
├── G trước nguyên âm hàng sau: a, ă, â, o, ô, ơ, u, ư
└── GH trước nguyên âm hàng trước: e, ê, i

VÍ DỤ:
├── ✓ gà, gỗ, gương
├── ✓ ghe, ghế, ghi
├── ✗ ge, gi (nếu muốn âm /ɣ/, phải viết: ghe, ghi)
└── Lưu ý: "gi" là phụ âm riêng, phát âm /z/ (Bắc) hoặc /j/ (Nam)
```

#### 4.4.3 Quy tắc NG / NGH

Tương tự g/gh, hai cách viết **ng** và **ngh** đều biểu thị âm vị /ŋ/:

| Chữ cái | Đứng trước nguyên âm | Ví dụ |
| ------- | -------------------- | ----- |
| **ng**  | a, ă, â, o, ô, ơ, u, ư | nga, ngăn, ngân, ngo, ngô, ngơ, ngu, ngư |
| **ngh** | e, ê, i              | nghe, nghề, nghĩ |

```
QUY TẮC:
├── NG trước nguyên âm hàng sau: a, ă, â, o, ô, ơ, u, ư
└── NGH trước nguyên âm hàng trước: e, ê, i

VÍ DỤ:
├── ✓ ngày, ngồi, ngủ, ngưng
├── ✓ nghe, nghề, nghỉ, nghĩa
├── ✗ nge, ngi (phải viết: nghe, nghi)
└── ✗ ngha, ngho (phải viết: nga, ngo)
```

#### 4.4.4 Tóm tắt Quy tắc Phân bố

```
┌───────────────────────────────────────────────────────────────┐
│           QUY TẮC PHÂN BỐ PHỤ ÂM THEO NGUYÊN ÂM              │
├───────────────────────────────────────────────────────────────┤
│                                                               │
│  Nguyên âm hàng sau    Nguyên âm hàng trước                  │
│  (a, ă, â, o, ô, ơ,    (e, ê, i, y)                          │
│   u, ư)                                                       │
│  ─────────────────     ─────────────────                      │
│        C         ←──────────→       K                         │
│        G         ←──────────→       GH                        │
│        NG        ←──────────→       NGH                       │
│                                                               │
│  QU: luôn đi cùng, không phân biệt nguyên âm sau             │
│                                                               │
└───────────────────────────────────────────────────────────────┘
```

#### 4.4.5 Ứng dụng cho Bộ gõ (IME Implications)

```
VALIDATION RULES:
│
├── Nếu buffer = "ce*" hoặc "ci*"
│   └── Không phải tiếng Việt (phải là "ke", "ki")
│
├── Nếu buffer = "ge*" hoặc "gi*" (với ý muốn âm /ɣ/)
│   └── Không hợp lệ (phải là "ghe", "ghi")
│   └── Lưu ý: "gi" hợp lệ nhưng là phụ âm /z/, không phải /ɣ/
│
├── Nếu buffer = "nge*" hoặc "ngi*"
│   └── Không hợp lệ (phải là "nghe", "nghi")
│
└── Nếu buffer = "gha*" hoặc "ngha*"
    └── Không hợp lệ (phải là "ga", "nga")
```

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

### 6.5 Ràng buộc Âm vị học (Phonotactic Constraints)

> **Mục đích**: Các quy tắc âm vị học xác định kết hợp hợp lệ - quan trọng cho validation trong bộ gõ.

#### 6.5.1 Cấm Cụm Phụ âm (No Consonant Clusters)

Tiếng Việt **KHÔNG** cho phép cụm phụ âm (consonant clusters) trong cùng một âm tiết:

```
KHÔNG HỢP LỆ - Các cụm phụ âm kiểu tiếng Anh/Pháp:
│
├── *l combinations: bl, cl, fl, gl, pl, sl
│   └── ✗ "black" → phải mượn: "blắc" hoặc "bờ-lắc"
│
├── *r combinations: br, cr, dr, fr, gr, pr, str
│   └── ✗ "press" → phải mượn: "prét" hoặc "pờ-rét"
│
├── s* combinations: sc, sk, sm, sn, sp, st, sw
│   └── ✗ "stop" → phải mượn: "xtốp"
│
├── *w combinations: dw, tw, sw
│   └── ✗ "dwell" → không có trong tiếng Việt
│
└── Cuối âm tiết: không có -nt, -nd, -lt, -lk, -mp, -sk...
    └── Chỉ có: -c, -ch, -m, -n, -ng, -nh, -p, -t
```

#### 6.5.2 Hạn chế P ở Đầu Âm tiết

Phụ âm **/p/** hầu như **KHÔNG** xuất hiện ở đầu từ thuần Việt:

```
P Ở ĐẦU:
├── Từ mượn: pin, pê-đan, pizza
├── Từ Hán-Việt: phòng, phố (viết PH, không phải P)
└── Từ thuần Việt: hầu như không có

P Ở CUỐI:
├── Hợp lệ: cấp, tập, lập, giúp, kịp
└── Chỉ với thanh sắc hoặc nặng
```

#### 6.5.3 Quy tắc Thanh điệu + Âm cuối Tắc

**Quy tắc quan trọng**: Âm tiết kết thúc bằng **p, t, c, ch** chỉ được mang **thanh sắc** hoặc **thanh nặng**:

```
┌────────────────────────────────────────────────────────────────┐
│              QUY TẮC THANH ĐIỆU + ÂM CUỐI TẮC                  │
├────────────────────────────────────────────────────────────────┤
│                                                                │
│  Âm cuối    Thanh hợp lệ       Thanh KHÔNG hợp lệ             │
│  ────────   ─────────────      ────────────────────           │
│  -p         sắc, nặng          ngang, huyền, hỏi, ngã         │
│  -t         sắc, nặng          ngang, huyền, hỏi, ngã         │
│  -c         sắc, nặng          ngang, huyền, hỏi, ngã         │
│  -ch        sắc, nặng          ngang, huyền, hỏi, ngã         │
│                                                                │
│  -m, -n     TẤT CẢ thanh       (không hạn chế)                │
│  -ng, -nh   TẤT CẢ thanh       (không hạn chế)                │
│  -i/y, -o/u TẤT CẢ thanh       (không hạn chế)                │
│                                                                │
└────────────────────────────────────────────────────────────────┘

VÍ DỤ:
├── ✓ cấp, cập (sắc, nặng + p)
├── ✓ mát, mạt (sắc, nặng + t)
├── ✓ các, cạc (sắc, nặng + c)
├── ✓ ách, ạch (sắc, nặng + ch)
│
├── ✗ cảp, cãp, cáp, càp (hỏi, ngã, ngang, huyền + p)
├── ✗ mảt, mãt, mat, màt (hỏi, ngã, ngang, huyền + t)
├── ✗ cảc, cãc, cac, càc (hỏi, ngã, ngang, huyền + c)
└── ✗ ảch, ãch, ach, àch (hỏi, ngã, ngang, huyền + ch)
```

#### 6.5.4 Ràng buộc Nguyên âm + Âm cuối

Không phải mọi nguyên âm đều kết hợp được với mọi âm cuối:

```
┌─────────────────────────────────────────────────────────────────┐
│              KẾT HỢP NGUYÊN ÂM + ÂM CUỐI                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Âm cuối -ch: chỉ sau a, ê, i                                  │
│  ├── ✓ ách, êch, ich (sách, bếch, ích)                        │
│  └── ✗ ôch, ơch, uch, ưch                                      │
│                                                                 │
│  Âm cuối -nh: chỉ sau a, ê, i, y                               │
│  ├── ✓ anh, ênh, inh, ynh (anh, bênh, xinh)                   │
│  └── ✗ ônh, ơnh, unh, ưnh                                      │
│                                                                 │
│  Âm cuối -ng: không sau e, ê                                   │
│  ├── ✓ ang, ăng, âng, ong, ông, ơng, ung, ưng                 │
│  └── ✗ eng, êng (dùng -nh thay: anh, ênh)                     │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

#### 6.5.5 Các Kết hợp KHÔNG HỢP LỆ (Invalid Combinations)

Danh sách tổng hợp các pattern không tồn tại trong tiếng Việt:

```
INVALID PATTERNS - Cho Bộ gõ:
│
├── PHỤ ÂM ĐẦU KHÔNG HỢP LỆ:
│   ├── Chữ không có: f, j, w, z (trừ từ mượn)
│   ├── Cụm phụ âm: bl, cl, fl, br, cr, dr, fr, gr, pr, st, sp...
│   └── Vi phạm c/k/g/gh/ng/ngh: ce, ci, ge(=ghe), nge, ngha...
│
├── THANH + ÂM CUỐI KHÔNG HỢP LỆ:
│   ├── hỏi/ngã/ngang/huyền + p: ảp, ãp, ap, àp
│   ├── hỏi/ngã/ngang/huyền + t: ảt, ãt, at, àt
│   ├── hỏi/ngã/ngang/huyền + c: ảc, ãc, ac, àc
│   └── hỏi/ngã/ngang/huyền + ch: ảch, ãch, ach, àch
│
├── NGUYÊN ÂM + ÂM CUỐI KHÔNG HỢP LỆ:
│   ├── ô/ơ/u/ư + ch: ôch, ơch, uch, ưch
│   ├── ô/ơ/u/ư + nh: ônh, ơnh, unh, ưnh
│   └── e/ê + ng: eng, êng
│
└── TRƯỜNG HỢP ĐẶC BIỆT:
    ├── "p" đầu từ thuần Việt: rất hiếm
    ├── "qu" không theo "u": qa, qe, qi (luôn phải là qu+nguyên âm)
    └── Nguyên âm ba: iêu, yêu, ươi, ươu, uôi, oai, oay, oeo, uây, uyê, uya, uyu, uêu, oao

INVALID DIPHTHONGS - Nguyên âm đôi không hợp lệ:
│
├── KHÔNG CÓ CIRCUMFLEX:
│   ├── ôa (chỉ có oa) - "ngoafo" phải cho "ngoào" không phải "ngồa"
│   ├── âi (chỉ có ai, ay)
│   └── ôe (chỉ có oe)
│
├── KHI ĐÃ CÓ DẤU THANH:
│   ├── V1 có dấu + V2 khác → KHÔNG áp circumflex cho V2
│   ├── Ví dụ: "tà" + "oo" → "tàoo" (không phải "tàô")
│   ├── Ví dụ: "tò" + "aa" → "tòaa" (không phải "tòâ")
│   └── Ví dụ: "mù" + "aa" → "mùaa" (không phải "mùâ")
│
└── NGOẠI LỆ CÙNG NGUYÊN ÂM:
    └── "đé" + "e" → "đến" (same vowel, cho phép circumflex)
```

#### 6.5.6 Bảng Tham chiếu Nhanh cho Validation

```rust
// Pseudo-code cho IME validation

fn is_valid_tone_final(tone: Tone, final_c: Option<&str>) -> bool {
    match final_c {
        // Âm cuối tắc: chỉ sắc hoặc nặng
        Some("p") | Some("t") | Some("c") | Some("ch") => {
            matches!(tone, Tone::Sac | Tone::Nang)
        }
        // Âm cuối khác: tất cả thanh đều OK
        _ => true
    }
}

fn is_valid_vowel_final(vowel: &str, final_c: &str) -> bool {
    match final_c {
        "ch" => matches!(vowel, "a" | "ă" | "ê" | "i"),
        "nh" => matches!(vowel, "a" | "ă" | "ê" | "i" | "y"),
        "ng" => !matches!(vowel, "e" | "ê"),  // e, ê dùng -nh
        _ => true
    }
}

fn is_valid_initial(initial: &str) -> bool {
    // Single consonants
    let single = ["b","c","d","đ","g","h","k","l","m","n","p","q","r","s","t","v","x"];
    // Double consonants
    let double = ["ch","gh","gi","kh","ng","nh","ph","qu","th","tr"];
    // Triple
    let triple = ["ngh"];

    single.contains(&initial) || double.contains(&initial) || triple.contains(&initial)
}
```

---

## 7. Quy tắc đặt dấu thanh

### 7.1 Nguyên tắc cơ bản

Dấu thanh đặt trên **nguyên âm chính** (âm chính) của âm tiết.

### 7.2 Quy tắc tổng quát

```
QUY TẮC ĐẶT DẤU THANH:

1. MỘT NGUYÊN ÂM: Dấu đặt trực tiếp trên nguyên âm đó
   VD: bá, bà, bả, bã, bạ

2. HAI/BA NGUYÊN ÂM: Xem Section 7.6 (Ma trận tổng hợp)
   - Tra cột "Dấu thanh" trong bảng 7.6.1

3. ƯU TIÊN DẤU PHỤ: Nếu có ă,â,ê,ô,ơ,ư → dấu thanh đặt trên nguyên âm đó
   VD: sứa (ư), đời (ơ), luật (â)
```

> **Chi tiết đầy đủ**: [Section 7.6 - Ma trận tổng hợp cho IME](#76-ma-trận-tổng-hợp-cho-ime-consolidated-ime-matrix)

### 7.4 Hai trường phái đặt dấu: Kiểu cũ và Kiểu mới

Hiện nay tồn tại 2 quan điểm về cách đặt dấu thanh, thường được gọi là "kiểu cũ" và "kiểu mới".

#### 7.4.1 Kiểu cũ

- **Cơ sở**: Dựa trên nhãn quan, giữ vị trí dấu ở giữa hoặc gần giữa từ cho cân bằng
- **Đặc điểm**: Coi "gi" và "qu" là một chữ cái riêng
  - "già" = "gi" + "à" (không phải nguyên âm đôi "ia")
  - "quạ" = "qu" + "ạ" (không phải nguyên âm đôi "ua")
- **Nguồn gốc**: Dựa trên các từ điển trước năm 1950

#### 7.4.2 Kiểu mới

- **Cơ sở**: Dựa trên ngữ âm học, đối chiếu chữ viết và âm thanh
- **Đặc điểm**: Đặt dấu thanh vào vị trí của âm chính theo ký hiệu ngữ âm quốc tế (IPA)
  - /wa/, /wɛ/, /wi/ → đặt dấu vào a, e, i
- **Quy định**: Theo Quyết định 1989/QĐ-BGDĐT ngày 25/5/2018 của Bộ GD&ĐT

#### 7.4.3 Bảng so sánh các trường hợp khác biệt

| Pattern | Kiểu cũ                   | Kiểu mới                  |
| ------- | ------------------------- | ------------------------- |
| oa      | hòa, hóa, hỏa, hõa, họa   | hoà, hoá, hoả, hoã, hoạ   |
| oe      | hòe, hóe, hỏe, hõe, họe   | hoè, hoé, hoẻ, hoẽ, hoẹ   |
| uy      | thùy, thúy, thủy, thũy, thụy | thuỳ, thuý, thuỷ, thuỹ, thuỵ |

#### 7.4.4 Tranh luận

| Quan điểm | Lập luận |
| --------- | -------- |
| Ủng hộ kiểu mới | Vì oa, oe, uy được ký âm IPA là /wa/, /wɛ/, /wi/ nên phải bỏ dấu vào a, e, i |
| Ủng hộ kiểu cũ | Ký hiệu IPA biểu thị cách phát âm, không phải cách viết |

> **Lưu ý**: Hiện tại cả hai kiểu đều được sử dụng song song trong tiếng Việt. Các bộ gõ như UniKey, EVKey đều cho phép người dùng chọn kiểu đặt dấu phù hợp.

### 7.5 Quy tắc đặt dấu phụ (Diacritic Placement)

Dấu phụ (circumflex ^, horn, breve ˘) được định nghĩa trong cột **"Dấu phụ (modifier)"** của Section 7.6.1.

**Patterns đặc biệt cần lưu ý:**

| Pattern | Input | Output | Dấu phụ | Ghi chú |
|---------|-------|--------|---------|---------|
| ươ | u+o+w | ươ | CẢ HAI ← horn | được, mười |
| uơ | u+o+w | uơ | CHỈ O ← horn (Issue #133) | huơ, khuơ |
| ươu | u+o+u+w | ươu | Chỉ u,o ← horn (u cuối giữ nguyên) | rượu, hươu |
| ưu | u+u+w | ưu | THỨ NHẤT ← horn | lưu, hưu |
| ưa | C+u+a+w | ưa | THỨ NHẤT ← horn (có C đầu) | mưa, cửa |

> **Issue #133**: Trong một số từ như "huơ", "khuơ", chỉ có 'o' nhận dấu móc, 'u' giữ nguyên.
> Đây là các từ đặc biệt cần xử lý riêng trong engine.

> **Chi tiết đầy đủ**: [Section 7.6.1 - Ma trận kết hợp nguyên âm](#761-ma-trận-kết-hợp-nguyên-âm-hợp-lệ-valid-vowel-combinations)

### 7.6 Ma trận tổng hợp cho IME (Consolidated IME Matrix)

> **Mục đích**: Ma trận này kết hợp tất cả quy tắc validation và đặt dấu thành một bảng duy nhất, phục vụ làm **single source of truth** cho code implementation.

#### 7.6.1 Ma trận kết hợp nguyên âm hợp lệ (Valid Vowel Combinations)

> **Cách đọc ma trận**: Mỗi hàng là một pattern hợp lệ. Cột thanh điệu (sắc→nặng) cho thấy **chính xác nguyên âm nào nhận dấu**.
>
> **Legend**: ^ = circumflex, ˘ = breve, ʼ = horn

**A. Nguyên âm đôi (Diphthongs) - 29 patterns**

| # | Pattern | Keys | Modifier | sắc | huyền | hỏi | ngã | nặng |
|--:|---------|------|----------|-----|-------|-----|-----|------|
| 1 | ai | A+I | - | **á**i | **à**i | **ả**i | **ã**i | **ạ**i |
| 2 | ao | A+O | - | **á**o | **à**o | **ả**o | **ã**o | **ạ**o |
| 3 | au | A+U | - | **á**u | **à**u | **ả**u | **ã**u | **ạ**u |
| 4 | ay | A+Y | - | **á**y | **à**y | **ả**y | **ã**y | **ạ**y |
| 5 | âu | A+U | a→â (^) | **ấ**u | **ầ**u | **ẩ**u | **ẫ**u | **ậ**u |
| 6 | ây | A+Y | a→â (^) | **ấ**y | **ầ**y | **ẩ**y | **ẫ**y | **ậ**y |
| 7 | eo | E+O | - | **é**o | **è**o | **ẻ**o | **ẽ**o | **ẹ**o |
| 8 | êu | E+U | e→ê (^) | **ế**u | **ề**u | **ể**u | **ễ**u | **ệ**u |
| 9 | ia | I+A | - | **í**a | **ì**a | **ỉ**a | **ĩ**a | **ị**a |
| 10 | iê | I+E | e→ê (^) | i**ế** | i**ề** | i**ể** | i**ễ** | i**ệ** |
| 11 | iu | I+U | - | **í**u | **ì**u | **ỉ**u | **ĩ**u | **ị**u |
| 12 | oa | O+A | - | o**á** | o**à** | o**ả** | o**ã** | o**ạ** |
| 13 | oă | O+A | a→ă (˘) | o**ắ** | o**ằ** | o**ẳ** | o**ẵ** | o**ặ** |
| 14 | oe | O+E | - | o**é** | o**è** | o**ẻ** | o**ẽ** | o**ẹ** |
| 15 | oi | O+I | - | **ó**i | **ò**i | **ỏ**i | **õ**i | **ọ**i |
| 16 | ôi | O+I | o→ô (^) | **ố**i | **ồ**i | **ổ**i | **ỗ**i | **ộ**i |
| 17 | ơi | O+I | o→ơ (ʼ) | **ớ**i | **ờ**i | **ở**i | **ỡ**i | **ợ**i |
| 18 | ua | U+A | (âm mở) | **ú**a | **ù**a | **ủ**a | **ũ**a | **ụ**a |
| 19 | ua (q-) | U+A | (sau q) | qu**á** | qu**à** | qu**ả** | qu**ã** | qu**ạ** |
| 19b | ua+C | U+A | (có phụ âm cuối) | mu**á**n | mu**à**n | mu**ả**n | mu**ã**n | mu**ạ**n |
| 20 | uâ | U+A | a→â (^) | u**ấ** | u**ầ** | u**ẩ** | u**ẫ** | u**ậ** |
| 21 | uê | U+E | e→ê (^) | u**ế** | u**ề** | u**ể** | u**ễ** | u**ệ** |
| 22 | ui | U+I | - | **ú**i | **ù**i | **ủ**i | **ũ**i | **ụ**i |
| 23 | uô | U+O | o→ô (^) | u**ố** | u**ồ** | u**ổ** | u**ỗ** | u**ộ** |
| 24 | uy | U+Y | - | u**ý** | u**ỳ** | u**ỷ** | u**ỹ** | u**ỵ** |
| 25 | ưa | U+A | u→ư (ʼ) | **ứ**a | **ừ**a | **ử**a | **ữ**a | **ự**a |
| 26 | ưi | U+I | u→ư (ʼ) | **ứ**i | **ừ**i | **ử**i | **ữ**i | **ự**i |
| 27 | ươ | U+O | u→ư, o→ơ (ʼʼ) | ư**ớ** | ư**ờ** | ư**ở** | ư**ỡ** | ư**ợ** |
| 28 | ưu | U+U | u₁→ư (ʼ) | **ứ**u | **ừ**u | **ử**u | **ữ**u | **ự**u |
| 29 | yê | Y+E | e→ê (^) | y**ế** | y**ề** | y**ể** | y**ễ** | y**ệ** |

**B. Nguyên âm ba (Triphthongs) - 14 patterns**

| # | Pattern | Keys | Modifier | sắc | huyền | hỏi | ngã | nặng |
|--:|---------|------|----------|-----|-------|-----|-----|------|
| 30 | iêu | I+E+U | e→ê (^) | i**ế**u | i**ề**u | i**ể**u | i**ễ**u | i**ệ**u |
| 31 | yêu | Y+E+U | e→ê (^) | y**ế**u | y**ề**u | y**ể**u | y**ễ**u | y**ệ**u |
| 32 | oai | O+A+I | - | o**á**i | o**à**i | o**ả**i | o**ã**i | o**ạ**i |
| 33 | oay | O+A+Y | - | o**á**y | o**à**y | o**ả**y | o**ã**y | o**ạ**y |
| 34 | oeo | O+E+O | - | o**é**o | o**è**o | o**ẻ**o | o**ẽ**o | o**ẹ**o |
| 35 | uây | U+A+Y | a→â (^) | u**ấ**y | u**ầ**y | u**ẩ**y | u**ẫ**y | u**ậ**y |
| 36 | uôi | U+O+I | o→ô (^) | u**ố**i | u**ồ**i | u**ổ**i | u**ỗ**i | u**ộ**i |
| 37 | uya | U+Y+A | - | u**ý**a | u**ỳ**a | u**ỷ**a | u**ỹ**a | u**ỵ**a |
| 38 | ươi | U+O+I | u→ư, o→ơ (ʼʼ) | ư**ớ**i | ư**ờ**i | ư**ở**i | ư**ỡ**i | ư**ợ**i |
| 39 | ươu | U+O+U | u₁→ư, o→ơ (ʼʼ) | ư**ớ**u | ư**ờ**u | ư**ở**u | ư**ỡ**u | ư**ợ**u |
| 40 | uyê | U+Y+E | e→ê (^) | uy**ế** | uy**ề** | uy**ể** | uy**ễ** | uy**ệ** |
| 41 | uyu | U+Y+U | - | u**ý**u | u**ỳ**u | u**ỷ**u | u**ỹ**u | u**ỵ**u |
| 42 | uêu | U+E+U | e→ê (^) | u**ế**u | u**ề**u | u**ể**u | u**ễ**u | u**ệ**u |
| 43 | oao | O+A+O | - | o**á**o | o**à**o | o**ả**o | o**ã**o | o**ạ**o |

> **Note**: Bold (**x**) = nguyên âm nhận dấu thanh. Patterns 18/19/19b (ua) có vị trí dấu phụ thuộc ngữ cảnh:
> - Pattern 18 (âm mở): dấu trên u (mùa, múa)
> - Pattern 19 (sau q): dấu trên a (quà, quá)
> - Pattern 19b (có phụ âm cuối): dấu trên a (chuẩn, thuận)
>
> **Added 2025-12**: Patterns 41-43 cho các từ đặc biệt:
> - **uyu** (#41): khuỷu (khuỷu tay - elbow)
> - **uêu** (#42): nguều (nguều ngoào - tangled/messy)
> - **oao** (#43): ngoào (nguều ngoào)

#### 7.6.2 Validation & Placement Rules

```
┌──────────────────────────────────────────────────────────────────────────┐
│                         IME VALIDATION FLOW                              │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  Input: V1 + V2 [+ V3]                                                  │
│                                                                          │
│  1. WHITELIST CHECK: Base keys có trong 7.6.1?                          │
│     ├── FOUND → VALID, tiếp tục Step 2                                  │
│     └── NOT FOUND → REJECT ngay (eg: eu, ou, yo → invalid)              │
│                                                                          │
│  2. MODIFIER CHECK: Tra cột "Dấu phụ" trong 7.6.1                       │
│     ├── Pattern yêu cầu modifier → phải có modifier                     │
│     └── E+U không có ^ → INVALID (phải là êu, không phải eu)            │
│                                                                          │
│  3. TONE PLACEMENT: Tra cột "Dấu thanh" trong 7.6.1                     │
│     ├── 1 = Nguyên âm đầu                                               │
│     ├── 2 = Nguyên âm thứ hai/giữa                                      │
│     └── 3 = Nguyên âm cuối (chỉ uyê)                                    │
│                                                                          │
│  4. DIACRITIC PLACEMENT: Tra cột "Dấu phụ" trong 7.6.1                  │
│     └── Áp dụng circumflex/horn/breve theo pattern                      │
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘
```

#### 7.6.3 Bảng tra nhanh

**A. Nguyên âm đôi: V1 → Valid V2**

| V1 (Base) | V2 hợp lệ |
|-----------|-----------|
| **A** | i, o, u, y |
| **Ă** | (không làm V1) |
| **Â** | u, y |
| **E** | o |
| **Ê** | u |
| **I** | a, ê, u |
| **O** | a, ă, e, i |
| **Ô** | i |
| **Ơ** | i |
| **U** | a, â, ê, i, o, ô, y |
| **Ư** | a, i, ơ, u |
| **Y** | ê |

**B. Nguyên âm ba: V1+V2 → Valid V3**

| V1+V2 | V3 hợp lệ |
|-------|-----------|
| **iê** | u |
| **yê** | u |
| **oa** | i, y |
| **oe** | o |
| **uâ** | y |
| **uô** | i |
| **uy** | a, ê |
| **ươ** | i, u |

> **Cách dùng**: Whitelist approach - pattern không có trong bảng → REJECT.

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

> **Issue #312**: Khi nguyên âm đã có dấu phụ (horn/circumflex/breve), gõ nguyên âm cùng loại tiếp theo sẽ **thêm nguyên âm thô**, không áp dụng dấu mũ.
>
> | Input    | Output | Giải thích                                    |
> |----------|--------|-----------------------------------------------|
> | chuwa    | chưa   | u+w → ư, sau đó +a                            |
> | chuwaa   | chưaa  | chưa + a → thêm 'a' thô (KHÔNG phải chưâ)     |
> | aaa      | âa     | aa → â, +a → thêm 'a' thô                     |
>
> Lý do: Nguyên âm đã biến đổi (ư, â, ă, ơ...) không trigger circumflex khi gõ nguyên âm gốc.

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

## 10.5 English Auto-Restore Patterns (Phục hồi từ tiếng Anh)

Khi người dùng gõ tiếng Anh bằng bộ gõ Telex, một số ký tự sẽ bị chuyển thành dấu tiếng Việt (s→sắc, f→huyền, etc.). Tính năng Auto-Restore phát hiện các pattern tiếng Anh và tự động phục hồi về text gốc.

### 10.5.1 Các Pattern Phát hiện Tiếng Anh

```
PATTERN 1: MODIFIER + CONSONANT
├── "text" → x + t → English (restore)
├── "expect" → x + p → English (restore)
└── Ngoại lệ: Modifier + sonorant (m,n,ng,nh) → Vietnamese (keep)
    └── "làm" = l + à + m → Vietnamese (không restore)

PATTERN 2: W Ở ĐẦU + CONSONANT
├── "water", "window", "world" → W + consonant → English
└── Ngoại lệ: "ưng", "ưn" → W + sonorant final → Vietnamese

PATTERN 3: EI VOWEL PAIR
├── "their", "weird" → ei + modifier → English
└── Vietnamese không có nguyên âm đôi "ei"

PATTERN 4: P INITIAL + AI PATTERN
├── "pair" = P + ai + r → English
└── P đầu từ hiếm trong tiếng Việt thuần

PATTERN 5: W AS FINAL
├── "raw", "law", "saw" → vowel + W → English
└── W cuối từ không có trong tiếng Việt

PATTERN 6: F INITIAL
├── "fix", "file", "focus" → F initial → English
└── Tiếng Việt dùng PH thay cho âm /f/

PATTERN 7: MODIFIER + K ENDING
├── "risk", "disk", "task" → modifier + K → English
├── Tiếng Việt chỉ có K cuối với breve: Đắk, Lắk
└── Ngoại lệ: B, L initial + K → ethnic minority (keep)
    └── "Busk" → "Búk" (Vietnamese ethnic minority)

PATTERN 8: DOUBLE VOWEL + TONE AT END
├── "looks", "took" → oo + k → English (restore)
└── Khác với Telex "aa" → "â" pattern
```

### 10.5.2 Invalid Diphthong Blocking

Khi nguyên âm V1 đã có dấu thanh, KHÔNG áp dụng circumflex cho nguyên âm V2 khác:

| Input (Telex) | Wrong | Correct | Reason |
|---------------|-------|---------|--------|
| tafoo | taồ | tàoo | à + ô invalid → skip circumflex |
| tefoo | teồ | tèoo | è + ô invalid → skip circumflex |
| tofaa | toầ | tòaa | ò + â invalid → skip circumflex |
| tofee | toề | tòee | ò + ê invalid → skip circumflex |
| tifaa | tiầ | tìaa | ì + â invalid → skip circumflex |
| mufaa | muầ | mùaa | ù + â invalid → skip circumflex |

**Ngoại lệ cùng nguyên âm**:
- "ddense" → "đến" (e + ê = same vowel, cho phép)

### 10.5.3 Ethnic Minority Words (Từ dân tộc thiểu số)

Một số từ dân tộc thiểu số sử dụng K làm phụ âm cuối:

| Input | Output | Note |
|-------|--------|------|
| ddawks | đắk | Đắk Lắk province |
| lawks | lắk | Lắk (part of Đắk Lắk) |
| Busk | Búk | Búk district |
| Kroong | Krông | Krông Búk district |

**Lưu ý**: D initial (disk, desk, dusk) được restore vì Đ trong tiếng Việt gõ là DD.

---

## 11. Tài liệu tham khảo

### 11.1 Wikipedia tiếng Việt

- [Chữ Quốc ngữ](https://vi.wikipedia.org/wiki/Chữ_Quốc_ngữ) - Hệ thống chữ viết tiếng Việt
- [Tiếng Việt](https://vi.wikipedia.org/wiki/Tiếng_Việt) - Ngôn ngữ tiếng Việt
- [Âm vị học tiếng Việt](https://vi.wikipedia.org/wiki/Âm_vị_học_tiếng_Việt) - Ngữ âm học tiếng Việt
- [Thanh điệu](https://vi.wikipedia.org/wiki/Thanh_điệu) - Thanh điệu trong ngôn ngữ
- [Chữ viết tiếng Việt](https://vi.wikipedia.org/wiki/Chữ_viết_tiếng_Việt) - Lịch sử chữ viết tiếng Việt
- [Quy tắc đặt dấu thanh của chữ Quốc ngữ](https://vi.wikipedia.org/wiki/Quy_tắc_đặt_dấu_thanh_của_chữ_Quốc_ngữ) - Quy tắc đặt dấu (kiểu cũ vs kiểu mới)
- [Bộ gõ tiếng Việt](https://vi.wikipedia.org/wiki/Bộ_gõ_tiếng_Việt) - Các bộ gõ tiếng Việt
- [Telex (kiểu gõ)](https://vi.wikipedia.org/wiki/Telex_(kiểu_gõ)) - Phương pháp gõ Telex
- [Tiếng Việt và máy tính](https://vi.wikipedia.org/wiki/Tiếng_Việt_và_máy_tính) - Tiếng Việt trên máy tính

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

> **Xem thêm**: [Thuật toán Validation](./validation-algorithm.md) - Chi tiết thuật toán xác nhận âm tiết tiếng Việt.

---

## Changelog

- **2025-12-21**: Bổ sung triphthongs và English auto-restore patterns
  - **Section 7.6.1-B**: Thêm 3 triphthongs mới (#41-43)
    - **uyu** (#41): khuỷu (khuỷu tay)
    - **uêu** (#42): nguều (nguều ngoào)
    - **oao** (#43): ngoào
  - **Section 6.5.5**: Thêm Invalid Diphthongs
    - Quy tắc: V1 có dấu + V2 khác → không áp circumflex
    - Examples: tafoo→tàoo, tefoo→tèoo, mufaa→mùaa
  - **Section 10.5**: NEW - English Auto-Restore Patterns
    - 8 patterns phát hiện tiếng Anh
    - Invalid diphthong blocking rules
    - Ethnic minority word exceptions (Búk, Đắk, Lắk)
    - Modifier + K ending detection (risk, disk, task)

- **2025-12-18**: Refactor toàn diện - Section 7.6 là SINGLE SOURCE OF TRUTH
  - **Section 7.6.1** redesign: Ma trận với cột thanh điệu tường minh (sắc, huyền, hỏi, ngã, nặng)
    - Mỗi ô hiển thị chính xác output với dấu (bold = nguyên âm nhận dấu)
    - **43 patterns** (29 diphthongs + 14 triphthongs)
    - Cột Modifier: chỉ rõ dấu phụ (^=circumflex, ˘=breve, ʼ=horn)
    - **Added**: uya (#37) - pattern cho từ "khuya"
  - **Section 7.6.2**: Validation & Placement Rules (flowchart)
  - **Section 7.6.3**: Bảng tra nhanh (V1→V2, V1+V2→V3)
  - **Loại bỏ các phần trùng lặp**:
    - Section 3.2.3, 3.4.2 → reference đến 7.6
    - Section 7.2, 7.3 (verbose) → rút gọn, reference đến 7.6
    - Section 7.5 → rút gọn, giữ pattern đặc biệt
  - **Whitelist approach**: Pattern không có trong 7.6.1 = REJECT

- **2025-12-16**: Mở rộng và hoàn thiện Section 3 + Section 7
  - **Section 3 (Hệ thống nguyên âm)**:
    - Phân loại nguyên âm đôi theo hướng phát âm (Rising/Falling/Medial)
    - Bổ sung pattern **ưu** (lưu, hưu, ngưu) - thiếu trong bản trước
    - Thêm danh sách các tổ hợp KHÔNG tồn tại (ou, yo, eu, ae, yi, wu)
    - Thêm ma trận kết hợp nguyên âm (V1 x V2)
    - Thêm bảng tra cứu nhanh cho bộ gõ
  - **Section 7 (Quy tắc đặt dấu thanh)**:
    - Thêm mục 7.3: Ma trận đặt dấu thanh (Tone Mark Placement Matrix)
    - Bảng 7.3.1: Nguyên âm đôi - Âm mở (28 patterns)
    - Bảng 7.3.2: Nguyên âm đôi - Âm đóng (9 patterns)
    - Bảng 7.3.3: Nguyên âm ba (10 patterns)
    - Bảng 7.3.4: Tra nhanh theo vị trí (1/2/3)
    - Bổ sung pattern **ưu** vào mục 7.2.3

- **2025-12-14**: Di chuyển Thuật toán Validation sang file riêng
  - Xóa mục 12 (Thuật toán Xác nhận Âm tiết) - đã có trong [validation-algorithm.md](./validation-algorithm.md)

- **2025-12-08**: Bổ sung Quy tắc Chính tả và Ràng buộc Âm vị học
  - Thêm mục 4.4: Quy tắc Chính tả Phụ âm (c/k/q, g/gh, ng/ngh)
  - Thêm mục 6.5: Ràng buộc Âm vị học (Phonotactic Constraints)
    - Cấm cụm phụ âm (no consonant clusters)
    - Hạn chế P ở đầu âm tiết
    - Quy tắc thanh điệu + âm cuối tắc (p,t,c,ch chỉ sắc/nặng)
    - Ràng buộc nguyên âm + âm cuối (-ch, -nh, -ng)
    - Danh sách các kết hợp không hợp lệ
  - Thêm pseudo-code validation functions cho bộ gõ

- **2025-12-07**: Bổ sung quy tắc đặt dấu thanh
  - Thêm mục 7.4: Hai trường phái đặt dấu (kiểu cũ vs kiểu mới)
  - Thêm bảng so sánh các trường hợp khác biệt (oa, oe, uy)
  - Thêm Quyết định 1989/QĐ-BGDĐT của Bộ GD&ĐT
  - Cập nhật tất cả links Wikipedia sang tiếng Việt (vi.wikipedia.org)

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
