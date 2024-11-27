// Oh no - i've just made an utils file!

export function getRequestHeaders() : HeadersInit {
    return {
        "Content-Type": "application/json",
        "Authorization": localStorage.getItem("apiToken") || ""
    };
}