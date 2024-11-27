import './assets/main.css'
import './assets/tailwind.css';

import { createApp } from 'vue'
import App from './App.vue'
import router from './router'
import { useRouter } from 'vue-router';
import { getRequestHeaders } from './utils';

const app = createApp(App)

app.use(router)

app.mount('#app')

function deauthenticated() {
    localStorage.removeItem('apiToken');

    router.replace("login");
}

function checkAuth() {
    const isAuthenticated = !!localStorage.getItem('apiToken'); 
    if (isAuthenticated)
    {
        const API_URL = import.meta.env.VITE_BACKEND_URL;
        fetch(`${API_URL}/logged_in`, {
            headers: getRequestHeaders()
        }).then(x => {
            if (x.status != 200) {
                console.log("Got a non 200 response from backend, assuming deauthentication.");
                deauthenticated();
            } 
          }).catch(x => {
            console.log("Got an error response from backend, assuming deauthentication.");
            deauthenticated();
          });
    }
    
    setTimeout(checkAuth, 30000);
}

checkAuth();