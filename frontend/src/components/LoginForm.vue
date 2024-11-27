<template>
  <div class="flex justify-center items-center min-h-screen bg-gray-900">
    <div class="bg-gray-800 p-8 rounded-lg shadow-md w-96">
      <h2 class="text-2xl font-bold text-white text-center mb-6">Login</h2>

      <!-- Error Message -->
      <div v-if="state.errorMessage" class="mb-4 text-red-500 text-center">
        {{ state.errorMessage }}
      </div>

      <form @submit.prevent="handleLogin">
        <!-- username Field -->
        <div class="mb-4">
          <label for="username" class="block text-gray-400 mb-1">username</label>
          <input
            id="username"
            type="username"
            v-model="state.username"
            class="w-full px-4 py-2 bg-gray-700 text-white border border-gray-600 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
            placeholder="Enter your username"
            required
          />
        </div>

        <!-- Password Field -->
        <div class="mb-6">
          <label for="password" class="block text-gray-400 mb-1">Password</label>
          <input
            id="password"
            type="password"
            v-model="state.password"
            class="w-full px-4 py-2 bg-gray-700 text-white border border-gray-600 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
            placeholder="Enter your password"
            required
          />
        </div>

        <!-- Login Button -->
        <button
          type="submit"
          class="w-full bg-blue-600 hover:bg-blue-700 text-white py-2 px-4 rounded-lg font-semibold focus:outline-none focus:ring-2 focus:ring-blue-500"
        >
          Login
        </button>
      </form>

      <!-- Additional Links -->
      <div class="mt-4 text-center">
        <a href="#" class="text-sm text-blue-400 hover:underline">Forgot password?</a>
      </div>
    </div>
  </div>
</template>


<script lang="ts">
import { reactive, ref } from "vue";
import { useRouter } from "vue-router";

const API_URL = import.meta.env.VITE_BACKEND_URL;

interface ILoginResponse {
  success: boolean,
  token?: string,
}

export default {
  name: "Login",
  async setup() {
    const state = reactive({
      username: "",
      password: "",
      errorMessage: "",
    });

    const router = useRouter();


    // Handle form submission
    const handleLogin = async (): Promise<void> => {
      let res = await fetch(`${API_URL}/sign_in`, {
        method: 'POST',
        mode: 'cors',
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          username: state.username,
          password: state.password
        })
      });

      let loginRes: ILoginResponse = await res.json();
      if (loginRes.success) {
        localStorage.setItem('apiToken', loginRes.token!);
        router.push({"name": "home"});
      } else {
        state.errorMessage = "Invalid username or password";
      }
    };

    return { state, handleLogin };
  },
};
</script>

<style scoped>
body {
  margin: 0;
  font-family: Arial, sans-serif;
}
</style>