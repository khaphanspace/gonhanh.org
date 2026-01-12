/**
 * Vanilla JS Script to Auto-Fill Product Hunt Submission Form for Easy AI
 *
 * Usage:
 * 1. Navigate to the product submission page
 * 2. Open browser console (F12)
 * 3. Copy and paste this entire script
 * 4. Press Enter to execute
 *
 * Note: File uploads (logo, screenshots) cannot be auto-filled due to browser security.
 */

(function() {
  'use strict';

  // Product information
  const productData = {
    name: 'Easy AI',
    tagline: 'Nền tảng AI-native lấy CDP làm lõi, cá nhân hóa mọi điểm chạm khách hàng',
    description: `Nền tảng AI-native lấy CDP (Customer Data Platform) làm lõi, giúp doanh nghiệp cá nhân hóa mọi điểm chạm khách hàng từ chat, email đến website. Giải pháp all-in-one cho ecommerce Việt Nam.

Tính năng nổi bật:
• CDP thông minh - Thu thập và thống nhất dữ liệu khách hàng từ mọi kênh, tạo hồ sơ 360° về từng khách hàng
• Cá nhân hóa AI - Tự động phân tích hành vi và sở thích để cá nhân hóa nội dung, gợi ý sản phẩm phù hợp với từng khách hàng
• Chăm sóc tự động - AI chatbot thông minh trả lời câu hỏi, tư vấn sản phẩm 24/7 trên website, Facebook, Zalo
• Gợi ý sản phẩm thông minh - Đề xuất sản phẩm đúng người đúng lúc dựa trên lịch sử mua hàng và hành vi duyệt web
• Email Marketing AI - Tự động tạo và gửi email cá nhân hóa theo hành trình khách hàng
• Website cá nhân hóa - Thay đổi nội dung, banner, sản phẩm hiển thị theo từng phân khúc khách hàng
• Analytics & Insights - Báo cáo chi tiết về hành vi khách hàng, hiệu quả chiến dịch, dự đoán xu hướng
• Tích hợp đa nền tảng - Kết nối dễ dàng với Shopify, WooCommerce, Haravan và các nền tảng ecommerce phổ biến

Biến data thành doanh thu với Easy AI - Giải pháp AI marketing automation được thiết kế riêng cho thị trường Việt Nam.`,
    links: [
      'https://geteasy.ai/',
      'https://easyaichat.app/'
    ],
    videoUrl: '',
    categoryId: '1', // Artificial Intelligence (hoặc '5' cho Marketing)
    pricingType: 'freemium', // Paid with free trial
    submitterType: 'maker' // Thay 'hunter' nếu bạn không phải maker
  };

  console.log('🚀 Starting form auto-fill for Easy AI...');

  // Helper function to trigger input events
  function triggerInput(element, value) {
    element.value = value;
    element.dispatchEvent(new Event('input', { bubbles: true }));
    element.dispatchEvent(new Event('change', { bubbles: true }));
  }

  // Helper function to click element
  function clickElement(element) {
    if (element) {
      element.click();
      return true;
    }
    return false;
  }

  // Fill basic text inputs
  function fillTextInputs() {
    const nameInput = document.getElementById('name');
    const taglineInput = document.getElementById('tagline');
    const descriptionInput = document.getElementById('description');

    if (nameInput) {
      triggerInput(nameInput, productData.name);
      console.log('✓ Product name filled');
    }

    if (taglineInput) {
      triggerInput(taglineInput, productData.tagline);
      console.log('✓ Tagline filled');
    }

    if (descriptionInput) {
      triggerInput(descriptionInput, productData.description);
      console.log('✓ Description filled');
    }
  }

  // Fill product links
  function fillLinks() {
    productData.links.forEach((link, index) => {
      const linkInput = document.querySelector(`input[name="links\\[${index}\\]"]`);
      if (linkInput && linkInput.type !== 'hidden') {
        triggerInput(linkInput, link);
        console.log(`✓ Link ${index + 1} filled: ${link}`);
      }
    });

    // Click "Add Link" button if we have more than 1 link
    if (productData.links.length > 1) {
      const addLinkBtn = Array.from(document.querySelectorAll('button'))
        .find(btn => btn.textContent.includes('Add Link'));

      if (addLinkBtn) {
        setTimeout(() => {
          clickElement(addLinkBtn);
          setTimeout(() => {
            const secondLinkInput = document.querySelector('input[name="links\\[1\\]"]');
            if (secondLinkInput && secondLinkInput.type !== 'hidden') {
              triggerInput(secondLinkInput, productData.links[1]);
              console.log(`✓ Second link added: ${productData.links[1]}`);
            }
          }, 100);
        }, 100);
      }
    }
  }

  // Fill video URL
  function fillVideoUrl() {
    const videoInput = document.getElementById('video_url');
    if (videoInput && productData.videoUrl) {
      triggerInput(videoInput, productData.videoUrl);
      console.log('✓ Video URL filled');
    }
  }

  // Select category (handle custom select component)
  function selectCategory() {
    // First try to set the hidden native select
    const nativeSelect = document.querySelector('select[name="category_id"]');
    if (nativeSelect) {
      nativeSelect.value = productData.categoryId;
      nativeSelect.dispatchEvent(new Event('change', { bubbles: true }));
    }

    // Then click the custom select trigger to open dropdown
    const selectTrigger = document.querySelector('button[role="combobox"][aria-controls*="select"]');
    if (selectTrigger) {
      clickElement(selectTrigger);

      // Wait for dropdown to appear, then select category
      setTimeout(() => {
        const options = document.querySelectorAll('[role="option"]');
        // Category ID 1 = Artificial Intelligence, 5 = Marketing
        const categoryOption = Array.from(options).find(
          opt => opt.textContent.includes('Artificial Intelligence') || opt.textContent.includes('Marketing')
        );

        if (categoryOption) {
          clickElement(categoryOption);
          console.log('✓ Category selected: ' + categoryOption.textContent);
        }
      }, 200);
    }
  }

  // Select pricing type (radio button)
  function selectPricingType() {
    const pricingRadio = document.getElementById(`pricing-${productData.pricingType}`);
    if (pricingRadio) {
      clickElement(pricingRadio);
      console.log(`✓ Pricing type selected: ${productData.pricingType}`);
    }

    // Also update the hidden input
    const hiddenInput = document.querySelector('input[name="pricing_type"][data-hidden]');
    if (hiddenInput) {
      hiddenInput.value = productData.pricingType;
    }
  }

  // Select submitter type (radio button)
  function selectSubmitterType() {
    const submitterRadio = document.getElementById(`submitter-${productData.submitterType}`);
    if (submitterRadio) {
      clickElement(submitterRadio);
      console.log(`✓ Submitter type selected: ${productData.submitterType}`);
    }

    // Also update the hidden input
    const hiddenInput = document.querySelector('input[name="submitter_type"][data-hidden]');
    if (hiddenInput) {
      hiddenInput.value = productData.submitterType;
    }
  }

  // Update character counts
  function updateCharacterCounts() {
    setTimeout(() => {
      const counters = document.querySelectorAll('.text-xs.text-muted-foreground');
      counters.forEach(counter => {
        const span = counter.querySelector('span');
        if (span) {
          const prevElement = counter.previousElementSibling;
          if (prevElement && (prevElement.tagName === 'INPUT' || prevElement.tagName === 'TEXTAREA')) {
            span.textContent = prevElement.value.length.toString();
          }
        }
      });
      console.log('✓ Character counts updated');
    }, 300);
  }

  // Execute all fill operations
  function fillForm() {
    fillTextInputs();
    fillLinks();
    fillVideoUrl();
    selectCategory();
    selectPricingType();
    selectSubmitterType();
    updateCharacterCounts();

    console.log('\n📝 Form auto-fill completed for Easy AI!');
    console.log('⚠️  Note: Please manually upload:');
    console.log('   - Logo (recommended: 240x240px)');
    console.log('   - Screenshots (optional: 1270x760px)');
    console.log('\n✅ Review the form and click "Submit Product" when ready.');
  }

  // Run the script
  try {
    fillForm();
  } catch (error) {
    console.error('❌ Error filling form:', error);
    console.log('Please check if you\'re on the correct page and try again.');
  }

})();
