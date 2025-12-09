mod common;
use common::{telex, vni};

#[test]
fn paragraph_telex() {
    let input = "Tooi ddax thwr rawts nhieuf boj gox tiengss Vietj treen macOS nhwng toan gawpj bug khos chiju. Gox treen Chrome thif bij dinhs chuwx “aa” thanhf “aâ”, gox www thif thanhf “ưưư”, vaof Claude Code thif lawpj kys tuwj lung tung, conf Google Docs thif cuwss mawts dawus giuwax chuwngf. Frustrated voo cungf neen tooi quyeets ddjnhj tuwj build Gox Nhanh - boj gox handle mwowjt maf ngay car nhuwngx tuwf khos nhw: giwowngf, khuyr tay, khuyeens khichs, chuyeenr ddoir, nguyeenj vongj, huyr hoaij, quynhf hoa, khoer khoawnr, loaf xoaf, nghieeng ngar. Giowf tooi cos ther thoair mais prompt Claude Code bawngf tiengss Vietj, soanr proposal hay update report maf koong stress veef typo nwax. DDungs nhw expect, deadline gawps maf gox sai hoaif thif burnout laf cais chawcs. Legit recommend cho anh em dev, xaif laf ghieenf luoon as! Neeus cos feedback gif thif inbox tooi qua nhatkha1407@gmail.com nha.";
    let expected = "Tôi đã thử rất nhiều bộ gõ tiếng Việt trên macOS nhưng toàn gặp bug khó chịu. Gõ trên Chrome thì bị dính chữ “aa” thành “aâ”, gõ www thì thành “ưưư”, vào Claude Code thì lặp ký tự lung tung, còn Google Docs thì cứ mất dấu giữa chừng. Frustrated vô cùng nên tôi quyết định tự build Gõ Nhanh - bộ gõ handle mượt mà ngay cả những từ khó như: giường, khuỷu tay, khuyến khích, chuyển đổi, nguyện vọng, huỷ hoại, quỳnh hoa, khoẻ khoắn, loà xoà, nghiêng ngả. Giờ tôi có thể thoải mái prompt Claude Code bằng tiếng Việt, soạn proposal hay update report mà không stress về typo nữa. Đúng như expect, deadline gấp mà gõ sai hoài thì burnout là cái chắc. Legit recommend cho anh em dev, xài là ghiền luôn á! Nếu có feedback gì thì inbox tôi qua nhatkha1407@gmail.com nha.";
    
    telex(&[(input, expected)]);
}

#[test]
fn paragraph_vni() {
    let input = "To6i dda4 thu73 ra61t nhie62u bo65 go4 tie61ng Vie65t tre6n macOS nhu7ng toa2n ga65p bug kho1 chi5u. Go4 tre6n Chrome thi2 bi5 di1nh chu74 “aa” tha2nh “aâ”, go4 www thi2 tha2nh “ưưư”, va2o Claude Code thi2 la65p ky1 tu75 lung tung, co2n Google Docs thi2 cu71 ma61t da61u giu74a chu72ng. Frustrated vo6 cu2ng ne6n to6i quye61t ddi5nh tu75 build Go4 Nhanh - bo65 go4 handle mu7o75t ma2 ngay ca3 nhu74ng tu72 kho1 nhu7: giu7o72ng, khuy3u tay, khuye61n khi1ch, chuye63n ddo63i, nguye65n vo5ng, huy3 hoa5i, quy2nh hoa, khoe3 khoa71n, loa2 xoa2, nghie6ng nga3. Gio72 to6i co1 the63 thoa3i ma1i prompt Claude Code ba72ng tie61ng Vie65t, soa5n proposal hay update report ma2 kho6ng stress ve62 typo nu7a4. DDu1ng nhu7 expect, deadline ga61p ma2 go4 sai hoa2i thi2 burnout la2 ca1i cha61c. Legit recommend cho anh em dev, xa2i la2 ghie62n luo6n a1! Ne61u co1 feedback gi2 thi2 inbox to6i qua nhatkha1407@gmail.com nha.";
    let expected = "Tôi đã thử rất nhiều bộ gõ tiếng Việt trên macOS nhưng toàn gặp bug khó chịu. Gõ trên Chrome thì bị dính chữ “aa” thành “aâ”, gõ www thì thành “ưưư”, vào Claude Code thì lặp ký tự lung tung, còn Google Docs thì cứ mất dấu giữa chừng. Frustrated vô cùng nên tôi quyết định tự build Gõ Nhanh - bộ gõ handle mượt mà ngay cả những từ khó như: giường, khuỷu tay, khuyến khích, chuyển đổi, nguyện vọng, huỷ hoại, quỳnh hoa, khoẻ khoắn, loà xoà, nghiêng ngả. Giờ tôi có thể thoải mái prompt Claude Code bằng tiếng Việt, soạn proposal hay update report mà không stress về typo nữa. Đúng như expect, deadline gấp mà gõ sai hoài thì burnout là cái chắc. Legit recommend cho anh em dev, xài là ghiền luôn á! Nếu có feedback gì thì inbox tôi qua nhatkha1407@gmail.com nha.";

    vni(&[(input, expected)]);
}
