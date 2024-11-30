<template>
    <div class="fixed inset-0 bg-black bg-opacity-50 flex justify-center items-center z-50">
        <div class="bg-gray-800 text-white rounded-lg shadow-lg p-6 w-3/4 max-w-2xl">
            <h2 class="text-2xl font-bold mb-4">Manage Timers</h2>

            <!-- Timer List -->
            <div>
                <h3 class="text-lg font-semibold mb-3">Existing Timers</h3>
                <ul v-if="timers.length" class="space-y-4">
                    <li v-for="timer in timers" :key="timer.id"
                        class="p-4 bg-gray-700 rounded-lg flex justify-between items-center">
                        <div>
                            <p>
                                <strong>Time:</strong> {{ formatTime(timer.startTime) }} - {{ formatTime(timer.endTime)
                                }}
                            </p>
                            <p>
                                <strong>Days:</strong> {{ formatDays(timer.days) }}
                            </p>
                            <p><strong>Status:</strong> {{ timer.isActive ? "Active" : "Inactive" }}</p>
                        </div>
                        <div class="flex space-x-2">
                            <button @click="editTimer(timer)" class="bg-blue-500 px-3 py-1 rounded hover:bg-blue-600">
                                Edit
                            </button>
                            <button @click="confirmDelete(timer.id)"
                                class="bg-red-500 px-3 py-1 rounded hover:bg-red-600">
                                Delete
                            </button>
                        </div>
                    </li>
                </ul>
                <p v-else class="text-gray-400">No timers found.</p>
            </div>

            <!-- Add/Edit Timer Form -->
            <div class="mt-6">
                <h3 class="text-lg font-semibold mb-3">
                    {{ isEditing ? "Edit Timer" : "Add Timer" }}
                </h3>
                <form @submit.prevent="saveTimer">
                    <div class="space-y-4">
                        <!-- Time Inputs -->
                        <div class="flex space-x-4">
                            <div>
                                <label class="block mb-1">Start Time</label>
                                <input type="time" v-model="form.startTime"
                                    class="w-full px-2 py-1 bg-gray-700 text-white border border-gray-600 rounded"
                                    required />
                            </div>
                            <div>
                                <label class="block mb-1">End Time</label>
                                <input type="time" v-model="form.endTime"
                                    class="w-full px-2 py-1 bg-gray-700 text-white border border-gray-600 rounded"
                                    required />
                            </div>
                        </div>

                        <!-- Days Selector -->
                        <div>
                            <label class="block mb-1">Days</label>
                            <div class="flex space-x-2">
                                <label v-for="(day, index) in daysOfWeek" :key="index"
                                    class="flex items-center space-x-1">
                                    <input type="checkbox" :value="index" v-model="form.days" class="text-blue-500" />
                                    <span>{{ day }}</span>
                                </label>
                            </div>
                        </div>

                        <!-- Active Toggle -->
                        <div>
                            <label class="flex items-center space-x-2">
                                <input type="checkbox" v-model="form.isActive" class="text-blue-500" />
                                <span>Active</span>
                            </label>
                        </div>

                        <div>
                            <label class="flex items-center space-x-2">
                                <input type="checkbox" v-model="form.oneOff" class="text-blue-500" />
                                <span>One-Off</span>
                            </label>
                        </div>
                    </div>

                    <!-- Action Buttons -->
                    <div class="mt-4 flex space-x-4">
                        <button type="submit" class="bg-green-500 hover:bg-green-600 px-4 py-2 rounded font-semibold">
                            Save
                        </button>
                        <button type="button" @click="resetForm"
                            class="bg-gray-600 hover:bg-gray-700 px-4 py-2 rounded font-semibold">
                            Cancel
                        </button>
                    </div>
                </form>
            </div>
        </div>
    </div>
</template>

<script lang="ts">
import { getRequestHeaders } from "@/utils";
import { defineComponent, ref, reactive } from "vue";

export interface ITimerInterface {
    id: number; // Unique identifier for the timer
    switchId: number; // ID of the associated switch
    startTime: number; // Start time in number of minutes from midnight
    endTime: number; // End time in number of minutes from midnight
    days: number[]; // Array of days (0=Monday, 6=Sunday)
    isActive: boolean; // Whether the timer is currently active
    oneOff: boolean; // Whether the timer is deactivated after the first time it fires
}

export default defineComponent({
    name: "TimerManagementModal",
    props: {
        switchId: { type: Number, required: true },
    },
    setup(props, { emit }) {
        const timers = reactive<ITimerInterface[]>([]);
        const isEditing = ref(false);
        const form = reactive({
            id: null as number | null,
            startTime: "",
            endTime: "",
            days: [] as number[],
            isActive: true,
            oneOff: false,
        });

        const daysOfWeek = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"];

        const fetchTimers = async () => {
            const API_URL = import.meta.env.VITE_BACKEND_URL;
            fetch(`${API_URL}/api/timers/${props.switchId}`, {
                headers: getRequestHeaders()
            }).then(x => x.json())
                .then((newTimers: ITimerInterface[]) => {
                    Object.assign(timers, newTimers);
                });

        };

        const timeToMinutes = (time: string): number => {
            const [hours, minutes] = time.split(":").map(Number);
            return hours * 60 + minutes;
        };

        const saveTimer = () => {
            const newTimer: ITimerInterface = {
                id: form.id ?? 0, 
                switchId: props.switchId,
                startTime: timeToMinutes(form.startTime),
                endTime: timeToMinutes(form.endTime),
                days: [...form.days],
                isActive: form.isActive,
                oneOff: form.oneOff
            };
            if (isEditing.value) {
                // Edit existing timer
                const index = timers.findIndex((t) => t.id === form.id);
                if (index !== -1) timers[index] = newTimer;
            } else {
                const API_URL = import.meta.env.VITE_BACKEND_URL;
                fetch(`${API_URL}/api/timer`, {
                    headers: getRequestHeaders(),
                    method: 'post',
                    body: JSON.stringify(newTimer)
                }).then(x => x.json())
                    .then(res => {
                        timers.push(newTimer);
                        fetchTimers();
                    });

            }
            resetForm();
        };

        const editTimer = (timer: ITimerInterface) => {
            isEditing.value = true;
            Object.assign(form, timer);
        };

        const confirmDelete = (id: number) => {
            if (confirm("Are you sure you want to delete this timer?")) {
                Object.assign(timers, timers.filter((t) => t.id !== id));
            }
        };

        const resetForm = () => {
            isEditing.value = false;
            Object.assign(form, {
                id: null,
                startTime: "",
                endTime: "",
                days: [],
                isActive: true,
            });
        };

        const formatTime = (minutes: number) => {
            const hours = Math.floor(minutes / 60)
                .toString()
                .padStart(2, "0");
            const mins = (minutes % 60).toString().padStart(2, "0");
            return `${hours}:${mins}`;
        };

        const formatDays = (days: number[]) => days.map((d) => daysOfWeek[d]).join(", ");

        fetchTimers(); // Load timers on mount

        return {
            timers,
            form,
            isEditing,
            daysOfWeek,
            saveTimer,
            editTimer,
            confirmDelete,
            resetForm,
            formatTime,
            formatDays,
        };
    },
});
</script>

<style scoped>
/* Add additional styles if needed */
</style>