const done = arguments[0]

const toRun = () => {
    swal("Error", "Bitte nur die Suchfunktion benutzen und auf ein Molekül klicken.\n\nSie werden auf die Startseite weitergeleitet.", "warning")
        .finally(() => done());
}

const exists = document.getElementById("script-sweetalert")
if(!exists) {
    const script = document.createElement("script")
    script.src = "https://unpkg.com/sweetalert/dist/sweetalert.min.js"
    script.id = "script-sweetalert"

    document.head.append(script)
    script.onload = () => toRun()
} else {
    toRun()
}