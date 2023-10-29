const done = arguments[0];

const div = document.querySelector(".grid-flow-col")

let wrapper = document.createElement("div");
wrapper.style = "grid-area: 2 / 3 / 3 / 1;width: 100%;"
wrapper.innerHTML = `<button id="page-generate-molecule-btn" class="text-sm flex w-full justify-center items-center gap-1 pc-gray-button">
<svg stroke="currentColor" fill="currentColor" stroke-width="0" viewBox="0 0 512 512" height="1em" width="1em" xmlns="http://www.w3.org/2000/svg"><path d="M132.172 157.504a155.154 155.154 0 0 0-18.296 21.698 99.274 99.274 0 1 1 186.291-53.827 153.447 153.447 0 0 0-58.134-12.138h-1.982a152.767 152.767 0 0 0-107.879 44.267zm105.97 263.021A153.877 153.877 0 0 1 93.014 311.583a99.286 99.286 0 1 0 162.84 108.154 155.965 155.965 0 0 1-15.719.8h-1.981zm125.101-231.262h-1.098a84.642 84.642 0 0 0-1.05 169.272h1.098a84.642 84.642 0 0 0 1.05-169.272zm-104.8 83.317a103.834 103.834 0 0 1 78.317-99.286 134.136 134.136 0 0 0-94.942-40.96h-1.743a134.566 134.566 0 0 0-1.67 269.107h1.742a133.993 133.993 0 0 0 85.31-30.53 103.917 103.917 0 0 1-67.014-98.33z"></path></svg>
<span class="ml-1">Generate Molecule Orbitals</span>
</button>`

div.appendChild(wrapper)

const button = document.getElementById("page-generate-molecule-btn")
button.onclick = () => {
    const element = document.createElement("div")
    element.style.display = "none"
    element.id = "calculator-molecule-confirmed"

    document.body.appendChild(element)
}
done()