function XIcon({ html, state }) {
    const { attrs={} } = state;
    const { size = "1em", color= "currentColor", aria_hidden="true" } = attrs;

    return html`
        <svg aria-hidden="${aria_hidden}" xmlns="http://www.w3.org/2000/svg" width=${size} height=${size} viewBox="0 0 24 24"><path fill="none" stroke=${color} stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M18 6L6 18M6 6l12 12" /></svg>
    `
}