function ToastMessage({ html, state }) {
    /* See 
        ./web/css/blocks/toast.css 
    */
    const { attrs={}, instanceID="" } = state;
    const { level, message, index } = attrs;
    
    return html`
        <style scope="component">
            .toast__content[data-toast-content-instance="${instanceID}"] {
                --toast-content-index: ${index};
            }
        </style>
        <article class="[ toast__content ] [ box ]" data-toast-content-instance="${instanceID}">
            <span class="[ toast__indicator ]" data-toast-indicator-level="${level}"></span>
            <p class="[ text-step-00 ]">${message}</p>
        </article>
    `
}