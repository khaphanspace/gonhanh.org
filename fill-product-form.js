/**
 * Vanilla JS Script to Auto-Fill Product Hunt Submission Form for Gõ Nhanh
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
    name: 'Gõ Nhanh',
    tagline: 'Bộ gõ tiếng Việt nhanh, ổn định cho macOS, Windows và Linux',
    description: `Bộ gõ tiếng Việt hiện đại giúp việc gõ tiếng Việt trở nên dễ dàng trên mọi nền tảng.

Tính năng nổi bật:
• Tự động khôi phục tiếng Anh - Thông minh phát hiện và sửa lỗi dấu tiếng Việt khi gõ từ tiếng Anh (ví dụ: "text" → "têt" → "text" khi nhấn space)
• Chuyển chế độ thông minh - Nhớ cài đặt BẬT/TẮT cho từng ứng dụng, tự động chuyển đổi khi bạn đổi ngữ cảnh
• Engine dựa trên âm vị học - Sử dụng cấu trúc ngữ âm tiếng Việt thay vì bảng tra cứu để gõ chính xác
• Cực kỳ nhanh - Độ trễ dưới 1ms với RAM ~5MB
• Nhân Rust thuần túy - Không phụ thuộc thư viện ngoài, đảm bảo ổn định và hiệu năng tối đa
• Đa nền tảng - Hỗ trợ macOS, Windows và Linux với tích hợp native
• Hỗ trợ Telex & VNI - Cả hai kiểu gõ tiếng Việt phổ biến đều có sẵn

Được xây dựng bởi một developer từng khó chịu với các bộ gõ hiện tại hay bị lỗi trên Chrome, VS Code và IDE search. Gõ Nhanh cung cấp trải nghiệm gõ tiếng Việt đáng tin cậy, thân thiện với developer.`,
    links: [
      'https://github.com/khaphanspace/gonhanh.org',
      'https://gonhanh.org'
    ],
    videoUrl: '',
    categoryId: '2', // Developer Tools
    pricingType: 'free',
    submitterType: 'maker'
  };

  console.log('🚀 Starting form auto-fill for Gõ Nhanh...');

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

      // Wait for dropdown to appear, then select "Developer Tools"
      setTimeout(() => {
        const options = document.querySelectorAll('[role="option"]');
        const developerToolsOption = Array.from(options).find(
          opt => opt.textContent.includes('Developer Tools')
        );

        if (developerToolsOption) {
          clickElement(developerToolsOption);
          console.log('✓ Category selected: Developer Tools');
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

    console.log('\n📝 Form auto-fill completed!');
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
