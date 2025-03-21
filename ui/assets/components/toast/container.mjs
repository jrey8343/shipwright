function ToastContainer({ html, state }) {
    /* See 
        ./web/css/blocks/toast.css 
    */
    const { attrs={}, instanceID='' } = state;
    const { count } = attrs;
    
    return html`
        <dialog class="[ toast ]" open data-toast-count="${count}" data-toast-instance="${instanceID}">
            <div class="[ stack ]">
                <slot></slot>
                <form method="dialog" hx-boost="false" class="[ toast__close ] [ cluster ]">
                    <button class="[ button ] [ cluster ]" data-button-variant="ghost" type="submit" data-toast-instance="${instanceID}">
                        Close<icon-x></icon-x>
                    </button>
                </form>     
            </div>
        </dialog>
        <script type="module">
            const toastContainer = document.querySelector('dialog[data-toast-instance="${instanceID}"]');
            // The toasts appear with the following delay  animation: slideUp 0.5s ease calc(var(--toast-index, 0) * 0.2s) forwards, slideOut 0.5s ease calc(5s + var(--toast-index, 0) * 0.2s) forwards;
            // Close the toastContainer after they have all run their course
            const button = toastContainer.querySelector('button[data-toast-instance="${instanceID}"]');
            const count = toastContainer.getAttribute('data-toast-count');
            const delay = 5000 + 500 + ((count - 1) * 2 * 200) + 200;
            const fadeOutTime = 200;  // Duration of the fade-out animation (in milliseconds)
            // Trigger fade out slightly before the toastContainer closes
            setTimeout(() => {
                button.classList.add('slide-out');
            }, delay - fadeOutTime);

            // Close the toastContainer after the delay
            setTimeout(() => toastContainer.close(), delay);
        </script>
    `
}