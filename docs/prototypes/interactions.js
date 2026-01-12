/**
 * Gõ Nhanh Interaction System
 * CJX-driven micro-interactions for prototype screens
 *
 * Design Philosophy:
 * - Subtle animations that don't distract
 * - Immediate feedback for user actions
 * - Accessibility-first (respects prefers-reduced-motion)
 */

// =============================================================================
// 1. CONFIGURATION
// =============================================================================

const InteractionConfig = {
  // Timing (ms)
  timing: {
    fast: 150,
    normal: 250,
    slow: 350,
  },

  // Easing functions
  easing: {
    ease: 'cubic-bezier(0.4, 0, 0.2, 1)',
    easeIn: 'cubic-bezier(0.4, 0, 1, 1)',
    easeOut: 'cubic-bezier(0, 0, 0.2, 1)',
    bounce: 'cubic-bezier(0.68, -0.55, 0.265, 1.55)',
  },

  // Check for reduced motion preference
  get prefersReducedMotion() {
    return window.matchMedia('(prefers-reduced-motion: reduce)').matches;
  },
};

// =============================================================================
// 2. UTILITY FUNCTIONS
// =============================================================================

/**
 * Animate an element with Web Animations API
 */
function animate(element, keyframes, options = {}) {
  if (InteractionConfig.prefersReducedMotion) {
    // Skip animation, apply final state immediately
    const finalFrame = keyframes[keyframes.length - 1];
    Object.assign(element.style, finalFrame);
    return Promise.resolve();
  }

  const defaultOptions = {
    duration: InteractionConfig.timing.normal,
    easing: InteractionConfig.easing.ease,
    fill: 'forwards',
  };

  return element.animate(keyframes, { ...defaultOptions, ...options }).finished;
}

/**
 * Debounce function calls
 */
function debounce(func, wait) {
  let timeout;
  return function executedFunction(...args) {
    const later = () => {
      clearTimeout(timeout);
      func(...args);
    };
    clearTimeout(timeout);
    timeout = setTimeout(later, wait);
  };
}

/**
 * Throttle function calls
 */
function throttle(func, limit) {
  let inThrottle;
  return function(...args) {
    if (!inThrottle) {
      func.apply(this, args);
      inThrottle = true;
      setTimeout(() => inThrottle = false, limit);
    }
  };
}

// =============================================================================
// 3. BUTTON INTERACTIONS
// =============================================================================

/**
 * Add press effect to buttons
 */
function initButtonInteractions() {
  document.querySelectorAll('.btn, button, [role="button"]').forEach(btn => {
    // Skip if already initialized
    if (btn.dataset.interactionInit) return;
    btn.dataset.interactionInit = 'true';

    btn.addEventListener('mousedown', () => {
      animate(btn, [
        { transform: 'scale(1)' },
        { transform: 'scale(0.97)' },
      ], { duration: InteractionConfig.timing.fast });
    });

    btn.addEventListener('mouseup', () => {
      animate(btn, [
        { transform: 'scale(0.97)' },
        { transform: 'scale(1)' },
      ], { duration: InteractionConfig.timing.fast });
    });

    btn.addEventListener('mouseleave', () => {
      btn.style.transform = 'scale(1)';
    });
  });
}

// =============================================================================
// 4. TOGGLE INTERACTIONS
// =============================================================================

/**
 * Add haptic-style feedback to toggles
 */
function initToggleInteractions() {
  document.querySelectorAll('.toggle input').forEach(toggle => {
    if (toggle.dataset.interactionInit) return;
    toggle.dataset.interactionInit = 'true';

    toggle.addEventListener('change', function() {
      const track = this.nextElementSibling;
      if (!track) return;

      // Haptic-style bounce
      animate(track, [
        { transform: 'scale(1)' },
        { transform: 'scale(0.9)' },
        { transform: 'scale(1.02)' },
        { transform: 'scale(1)' },
      ], {
        duration: InteractionConfig.timing.normal,
        easing: InteractionConfig.easing.bounce,
      });

      // Emit custom event
      this.dispatchEvent(new CustomEvent('toggle:changed', {
        bubbles: true,
        detail: { checked: this.checked },
      }));
    });
  });
}

// =============================================================================
// 5. LIST ITEM INTERACTIONS
// =============================================================================

/**
 * Add hover/active states to list items
 */
function initListInteractions() {
  document.querySelectorAll('.list-item, .menu-item').forEach(item => {
    if (item.dataset.interactionInit) return;
    item.dataset.interactionInit = 'true';

    item.addEventListener('mouseenter', () => {
      animate(item, [
        { backgroundColor: 'transparent' },
        { backgroundColor: 'var(--color-surface)' },
      ], { duration: InteractionConfig.timing.fast });
    });

    item.addEventListener('mouseleave', () => {
      animate(item, [
        { backgroundColor: 'var(--color-surface)' },
        { backgroundColor: 'transparent' },
      ], { duration: InteractionConfig.timing.fast });
    });

    // Click ripple effect
    item.addEventListener('click', function(e) {
      const ripple = document.createElement('div');
      ripple.style.cssText = `
        position: absolute;
        border-radius: 50%;
        background: var(--color-primary);
        opacity: 0.2;
        transform: scale(0);
        pointer-events: none;
        width: 100px;
        height: 100px;
        left: ${e.offsetX - 50}px;
        top: ${e.offsetY - 50}px;
      `;

      this.style.position = 'relative';
      this.style.overflow = 'hidden';
      this.appendChild(ripple);

      animate(ripple, [
        { transform: 'scale(0)', opacity: 0.3 },
        { transform: 'scale(4)', opacity: 0 },
      ], { duration: 400 }).then(() => ripple.remove());
    });
  });
}

// =============================================================================
// 6. CARD INTERACTIONS
// =============================================================================

/**
 * Add subtle lift effect to cards
 */
function initCardInteractions() {
  document.querySelectorAll('.card, .section').forEach(card => {
    if (card.dataset.interactionInit) return;
    card.dataset.interactionInit = 'true';

    card.addEventListener('mouseenter', () => {
      if (InteractionConfig.prefersReducedMotion) return;
      card.style.transition = `box-shadow ${InteractionConfig.timing.fast}ms`;
      card.style.boxShadow = 'var(--shadow-md)';
    });

    card.addEventListener('mouseleave', () => {
      card.style.boxShadow = '';
    });
  });
}

// =============================================================================
// 7. FORM INPUT INTERACTIONS
// =============================================================================

/**
 * Add focus animations to inputs
 */
function initInputInteractions() {
  document.querySelectorAll('.input, input[type="text"]').forEach(input => {
    if (input.dataset.interactionInit) return;
    input.dataset.interactionInit = 'true';

    input.addEventListener('focus', () => {
      animate(input, [
        { borderColor: 'var(--color-border)' },
        { borderColor: 'var(--color-border-focus)' },
      ], { duration: InteractionConfig.timing.fast });
    });

    input.addEventListener('blur', () => {
      animate(input, [
        { borderColor: 'var(--color-border-focus)' },
        { borderColor: 'var(--color-border)' },
      ], { duration: InteractionConfig.timing.fast });
    });
  });
}

// =============================================================================
// 8. MODAL/DIALOG INTERACTIONS
// =============================================================================

/**
 * Initialize modal open/close animations
 */
function initModalInteractions() {
  // Open modal function
  window.openModal = function(modalId) {
    const overlay = document.getElementById(modalId);
    if (!overlay) return;

    overlay.style.display = 'flex';
    const dialog = overlay.querySelector('.dialog');

    animate(overlay, [
      { opacity: 0 },
      { opacity: 1 },
    ], { duration: InteractionConfig.timing.normal });

    if (dialog) {
      animate(dialog, [
        { opacity: 0, transform: 'scale(0.95) translateY(-10px)' },
        { opacity: 1, transform: 'scale(1) translateY(0)' },
      ], {
        duration: InteractionConfig.timing.normal,
        easing: InteractionConfig.easing.easeOut,
      });
    }
  };

  // Close modal function
  window.closeModal = function(modalId) {
    const overlay = document.getElementById(modalId);
    if (!overlay) return;

    const dialog = overlay.querySelector('.dialog');

    const closeAnimation = animate(overlay, [
      { opacity: 1 },
      { opacity: 0 },
    ], { duration: InteractionConfig.timing.fast });

    if (dialog) {
      animate(dialog, [
        { opacity: 1, transform: 'scale(1) translateY(0)' },
        { opacity: 0, transform: 'scale(0.95) translateY(-10px)' },
      ], { duration: InteractionConfig.timing.fast });
    }

    closeAnimation.then(() => {
      overlay.style.display = 'none';
    });
  };

  // Close on overlay click
  document.querySelectorAll('.dialog-overlay').forEach(overlay => {
    overlay.addEventListener('click', function(e) {
      if (e.target === this) {
        closeModal(this.id);
      }
    });
  });

  // Close on ESC
  document.addEventListener('keydown', (e) => {
    if (e.key === 'Escape') {
      const openModal = document.querySelector('.dialog-overlay[style*="display: flex"]');
      if (openModal) {
        closeModal(openModal.id);
      }
    }
  });
}

// =============================================================================
// 9. WINDOW CHROME INTERACTIONS
// =============================================================================

/**
 * Window controls hover effects
 */
function initWindowInteractions() {
  document.querySelectorAll('.window-control').forEach(control => {
    if (control.dataset.interactionInit) return;
    control.dataset.interactionInit = 'true';

    control.addEventListener('mouseenter', () => {
      animate(control, [
        { transform: 'scale(1)' },
        { transform: 'scale(1.1)' },
      ], { duration: InteractionConfig.timing.fast });
    });

    control.addEventListener('mouseleave', () => {
      animate(control, [
        { transform: 'scale(1.1)' },
        { transform: 'scale(1)' },
      ], { duration: InteractionConfig.timing.fast });
    });
  });
}

// =============================================================================
// 10. NOTIFICATION TOAST
// =============================================================================

/**
 * Show a toast notification
 */
window.showToast = function(message, type = 'info', duration = 3000) {
  const toast = document.createElement('div');
  toast.className = `toast toast-${type}`;
  toast.textContent = message;
  toast.style.cssText = `
    position: fixed;
    bottom: 20px;
    left: 50%;
    transform: translateX(-50%) translateY(100px);
    padding: 12px 24px;
    background: var(--color-text-primary);
    color: var(--color-background);
    border-radius: 8px;
    font-size: 13px;
    z-index: var(--z-notification);
    box-shadow: var(--shadow-lg);
  `;

  document.body.appendChild(toast);

  // Animate in
  animate(toast, [
    { transform: 'translateX(-50%) translateY(100px)', opacity: 0 },
    { transform: 'translateX(-50%) translateY(0)', opacity: 1 },
  ], {
    duration: InteractionConfig.timing.normal,
    easing: InteractionConfig.easing.easeOut,
  });

  // Animate out after duration
  setTimeout(() => {
    animate(toast, [
      { transform: 'translateX(-50%) translateY(0)', opacity: 1 },
      { transform: 'translateX(-50%) translateY(100px)', opacity: 0 },
    ], {
      duration: InteractionConfig.timing.normal,
      easing: InteractionConfig.easing.easeIn,
    }).then(() => toast.remove());
  }, duration);
};

// =============================================================================
// 11. STEP TRANSITIONS (Onboarding)
// =============================================================================

/**
 * Animate between onboarding steps
 */
window.animateStepTransition = function(fromStep, toStep, direction = 'forward') {
  const from = document.querySelector(`[data-step="${fromStep}"]`);
  const to = document.querySelector(`[data-step="${toStep}"]`);

  if (!from || !to) return;

  const xOffset = direction === 'forward' ? -30 : 30;

  // Animate out
  animate(from, [
    { opacity: 1, transform: 'translateX(0)' },
    { opacity: 0, transform: `translateX(${-xOffset}px)` },
  ], { duration: InteractionConfig.timing.normal }).then(() => {
    from.classList.remove('active');
  });

  // Animate in
  to.classList.add('active');
  animate(to, [
    { opacity: 0, transform: `translateX(${xOffset}px)` },
    { opacity: 1, transform: 'translateX(0)' },
  ], { duration: InteractionConfig.timing.normal });
};

// =============================================================================
// 12. INITIALIZATION
// =============================================================================

/**
 * Initialize all interactions
 */
function initAllInteractions() {
  initButtonInteractions();
  initToggleInteractions();
  initListInteractions();
  initCardInteractions();
  initInputInteractions();
  initModalInteractions();
  initWindowInteractions();

  console.log('[Interactions] All interactions initialized');
}

// Initialize on DOM ready
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', initAllInteractions);
} else {
  initAllInteractions();
}

// Re-initialize when new content is added (for dynamic content)
const observer = new MutationObserver(debounce(() => {
  initAllInteractions();
}, 100));

observer.observe(document.body, {
  childList: true,
  subtree: true,
});

// Export for module usage
if (typeof module !== 'undefined' && module.exports) {
  module.exports = {
    InteractionConfig,
    animate,
    showToast,
    openModal,
    closeModal,
    animateStepTransition,
  };
}
