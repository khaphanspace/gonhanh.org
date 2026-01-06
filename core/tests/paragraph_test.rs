mod common;
use common::{telex_auto_restore, vni};

#[test]
fn paragraph_telex() {
    // Telex patterns from typing_test.rs:
    // - â = aa, ê = ee, ô = oo
    // - ă = aw, ơ = ow, ư = uw (after consonant)
    // - iê = iee, ươ = uwow
    // - đ = dd
    // - sắc/huyền/hỏi/ngã/nặng = s/f/r/x/j
    //
    // Auto-restore feature: Invalid Vietnamese words are auto-restored to English
    // when space/break key is pressed:
    // - "Google" → initially "Gôgle" (oo→ô) but invalid Vietnamese → restored to "Google"
    // - "expect" → initially "ễpct" (x→ngã, ee→ê) but invalid → restored to "expect"
    // - "burnout" → initially "bủnout" (r→hỏi, ou invalid) → restored to "burnout"
    // - "Docs" → "Dóc" (s→sắc) is VALID Vietnamese structure (D+ó+c) → stays transformed
    // - "deadline" stays as "deadline" (ea is invalid Vietnamese pattern, no transform applied)
    //
    // Issue #51: "deadline" now stays as "deadline" because the 'd's are not adjacent.
    // In Telex, "dd" → "đ" only applies when the two 'd's are consecutive.
    //
    // www behavior: w→ư, ww→w (revert), www→ww (subsequent w added normally)
    let input = "Tooi ddax thuwr raats nhieeuf booj gox tieengs Vieetj treen macOS nhuwng toanf gawpj bug khos chiuj. Gox treen Chrome thif bij dinhs chuwx \"aaa\" thanhf \"aâ\", gox www thif thanhf \"ưưư\", vaof Claude Code thif lawpj kys tuwj lung tung, conf Google Docs thif cuws maats daaus giuwax chuwngf. Frustrated voo cungf neen tooi quyeets ddinhj tuwj build Gox Nhanh - booj gox handle muwowjt maf ngay car nhuwngx tuwf khos nhuw: giuwowngf, khuyru tay, khuyeens khichs, chuyeenr ddooir, nguyeenj vongj, huyr hoaij, quynhf hoa, khoer khoawns, loaf xoaf, nghieeng ngar. Giowf tooi cos theer thoair mais prompt Claude Code bawngf tieengs Vieetj, soanj proposal hay update report maf khoong stress veef typo nuwax. DDungs nhuw expect, deadline gaaps maf gox sai hoaif thif burnout laf cais chawcs. Legit recommend cho anh em dev, xaif laf ghieenf luoon as! Neeus cos feedback gif thif inbox tooi qua nhatkha1407@gmail.com nha.";
    let expected = "Tôi đã thử rất nhiều bộ gõ tiếng Việt trên macOS nhưng toàn gặp bug khó chịu. Gõ trên Chrome thì bị dính chữ \"aa\" thành \"aâ\", gõ ww thì thành \"ưưư\", vào Claude Code thì lặp ký tự lung tung, còn Google Dóc thì cứ mất dấu giữa chừng. Frustrated vô cùng nên tôi quyết định tự build Gõ Nhanh - bộ gõ handle mượt mà ngay cả những từ khó như: giường, khuỷu tay, khuyến khích, chuyển đổi, nguyện vọng, huỷ hoại, quỳnh hoa, khoẻ khoắn, loà xoà, nghiêng ngả. Giờ tôi có thể thoải mái prompt Claude Code bằng tiếng Việt, soạn proposal hay update report mà không stress về typo nữa. Đúng như expect, deadline gấp mà gõ sai hoài thì burnout là cái chắc. Legit recommend cho anh em dev, xài là ghiền luôn á! Nếu có feedback gì thì inbox tôi qua nhatkha1407@gmail.com nha.";

    telex_auto_restore(&[(input, expected)]);
}

#[test]
fn paragraph_vni() {
    // VNI patterns:
    // - 6 = circumflex (â, ê, ô), 7 = horn (ư, ơ), 8 = breve (ă)
    // - 9 = stroke (đ)
    // - 1/2/3/4/5 = sắc/huyền/hỏi/ngã/nặng
    //
    // Note: In VNI mode, foreign words like Google, expect, deadline, burnout are typed
    // literally and most stay unchanged because VNI uses numbers for modifiers.
    let input = "To6i d9a4 thu73 ra61t nhie62u bo65 go4 tie61ng Vie65t tre6n macOS nhu7ng toa2n ga85p bug kho1 chi5u. Go4 tre6n Chrome thi2 bi5 di1nh chu74 \"aa\" tha2nh \"aâ\", go4 www thi2 tha2nh \"ưưư\", va2o Claude Code thi2 la85p ky1 tu75 lung tung, co2n Google Docs thi2 cu71 ma61t da61u giu74a chu72ng. Frustrated vo6 cu2ng ne6n to6i quye61t d9i5nh tu75 build Go4 Nhanh - bo65 go4 handle mu7o75t ma2 ngay ca3 nhu74ng tu72 kho1 nhu7: giu7o72ng, khuy3u tay, khuye61n khi1ch, chuye63n d9o63i, nguye65n vo5ng, huy3 hoa5i, quy2nh hoa, khoe3 khoa81n, loa2 xoa2, nghie6ng nga3. Gio72 to6i co1 the63 thoa3i ma1i prompt Claude Code ba82ng tie61ng Vie65t, soa5n proposal hay update report ma2 kho6ng stress ve62 typo nu7a4. D9u1ng nhu7 expect, deadline ga61p ma2 go4 sai hoa2i thi2 burnout la2 ca1i cha81c. Legit recommend cho anh em dev, xa2i la2 ghie62n luo6n a1! Ne61u co1 feedback gi2 thi2 inbox to6i qua nhatkha1407@gmail.com nha.";
    let expected = "Tôi đã thử rất nhiều bộ gõ tiếng Việt trên macOS nhưng toàn gặp bug khó chịu. Gõ trên Chrome thì bị dính chữ \"aa\" thành \"aâ\", gõ www thì thành \"ưưư\", vào Claude Code thì lặp ký tự lung tung, còn Google Docs thì cứ mất dấu giữa chừng. Frustrated vô cùng nên tôi quyết định tự build Gõ Nhanh - bộ gõ handle mượt mà ngay cả những từ khó như: giường, khuỷu tay, khuyến khích, chuyển đổi, nguyện vọng, huỷ hoại, quỳnh hoa, khoẻ khoắn, loà xoà, nghiêng ngả. Giờ tôi có thể thoải mái prompt Claude Code bằng tiếng Việt, soạn proposal hay update report mà không stress về typo nữa. Đúng như expect, deadline gấp mà gõ sai hoài thì burnout là cái chắc. Legit recommend cho anh em dev, xài là ghiền luôn á! Nếu có feedback gì thì inbox tôi qua nhatkha1407@gmail.com nha.";

    vni(&[(input, expected)]);
}

#[test]
fn paragraph_smart_auto_restore() {
    // Comprehensive test for smart auto-restore feature
    // Tests: Vietnamese conversion, English preservation, ethnic minority place names,
    // double/triple letter handling
    let input = "Chafo cacs banfj, minhf ddang tesst Gox Nhanh. Smart auto restore: text, expect, perfect, window, with, their, wow, luxury, tesla, life, issue, feature, express, wonderful, support, core, care, saas, sax, push, work, hard, user. Per app memory: VS Code, Slack. Auto disable: Japanese, Korean, Chinese. DDawsk Lawsk, DDawsk Noong, Kroong Buks. Thanks for your wonderful support with thiss software.";
    let expected = "Chào các bạn, mình đang test Gõ Nhanh. Smart auto restore: text, expect, perfect, window, with, their, wow, luxury, tesla, life, issue, feature, express, wonderful, support, core, care, saas, sax, push, work, hard, user. Per app memory: VS Code, Slack. Auto disable: Japanese, Korean, Chinese. Đắk Lắk, Đắk Nông, Krông Búk. Thanks for your wonderful support with this software.";

    telex_auto_restore(&[(input, expected)]);
}
