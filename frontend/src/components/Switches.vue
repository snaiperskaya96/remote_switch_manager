<template>
    <div class="min-h-screen bg-gray-900 p-6 text-white">
        <h1 class="text-3xl font-bold text-center mb-6">Switch List</h1>

        <ul class="space-y-4">
            <li v-for="switchItem in switches" :key="switchItem.id"
                class="flex items-center justify-between bg-gray-800 p-4 rounded-lg shadow-md">
                <div>
                    <p class="text-lg font-medium">{{ switchItem.alias }}</p>
                    <p class="text-sm text-gray-400">ID: {{ switchItem.id }}</p>
                </div>

                <div class="flex items-center space-x-4">
                    <!-- Toggle Switch -->
                    <label class="inline-flex items-center cursor-pointer">
                        <span class="mr-2 text-sm text-gray-300">{{ switchItem.status.type }}</span>
                        <input type="checkbox" class="hidden" :checked="switchItem.status.type === 'On'"
                            @change="toggleSwitch(switchItem.id)" />
                        <div :class="[
                            'w-10 h-5 flex items-center bg-gray-600 rounded-full p-1 transition-colors duration-300',
                            switchItem.status.type === 'On' ? 'bg-green-500' : switchItem.status.type === 'Off' ? 'bg-gray-600' : 'bg-yellow-500'
                        ]">
                            <div :class="[
                                'w-4 h-4 bg-white rounded-full shadow-md transform transition-transform duration-300',
                                switchItem.status.type === 'On' ? 'translate-x-5' : 'translate-x-0'
                            ]"></div>
                        </div>
                    </label>

                    <!-- Menu Button -->
                    <button class="relative px-3 py-2 bg-gray-700 hover:bg-gray-600 rounded-lg"
                        @click="openMenu(switchItem.id)">
                        •••
                    </button>

                    <!-- Dropdown Menu -->
                    <div v-if="activeMenu === switchItem.id"
                        class="absolute right-0 mt-2 bg-gray-800 border border-gray-700 rounded-lg shadow-lg p-2 z-10">
                        <button class="block w-full text-left px-4 py-2 text-sm hover:bg-gray-700"
                            @click="openEditModal(switchItem)">
                            Edit
                        </button>
                        <button class="block w-full text-left px-4 py-2 text-sm hover:bg-gray-700"
                            @click="openTimerModal(switchItem)">
                            Open Timer Manager
                        </button>
                        <button class="block w-full text-left px-4 py-2 text-sm hover:bg-gray-700 text-red-400"
                            @click="openDeleteModal(switchItem.id)">
                            Delete
                        </button>
                    </div>
                </div>
            </li>
        </ul>

        <!-- Edit Modal -->
        <div v-if="modals.edit" class="fixed inset-0 flex items-center justify-center bg-black bg-opacity-50">
            <div class="bg-gray-800 p-6 rounded-lg w-96">
                <h2 class="text-xl font-bold mb-4">Edit Switch</h2>
                <form @submit.prevent="saveEdit">
                    <label for="editAlias" class="block text-gray-400 mb-1">Alias</label>
                    <input id="editAlias" type="text" v-model="editSwitch.alias"
                        class="w-full px-4 py-2 bg-gray-700 text-white border border-gray-600 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                        required />
                    <div class="flex justify-end mt-4">
                        <button type="button" class="bg-gray-600 hover:bg-gray-500 text-white px-4 py-2 rounded-lg mr-2"
                            @click="closeModal('edit')">
                            Cancel
                        </button>
                        <button type="submit" class="bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-lg">
                            Save
                        </button>
                    </div>
                </form>
            </div>
        </div>

        <!-- Delete Modal -->
        <div v-if="modals.delete" class="fixed inset-0 flex items-center justify-center bg-black bg-opacity-50">
            <div class="bg-gray-800 p-6 rounded-lg w-96 text-center">
                <h2 class="text-xl font-bold mb-4">Delete Switch</h2>
                <p class="text-gray-400 mb-6">Are you sure you want to delete this switch?</p>
                <div class="flex justify-center space-x-4">
                    <button class="bg-gray-600 hover:bg-gray-500 text-white px-4 py-2 rounded-lg"
                        @click="closeModal('delete')">
                        Cancel
                    </button>
                    <button class="bg-red-600 hover:bg-red-700 text-white px-4 py-2 rounded-lg" @click="deleteSwitch">
                        Delete
                    </button>
                </div>
            </div>
        </div>

        <!-- Timer Manager Modal -->
        <TimerManagementModal v-if="modals.timer" :switchId="editSwitch.id" />
    </div>
</template>

<script lang="ts">
import { defineComponent, reactive, ref } from "vue";
import TimerManagementModal from "./TimerManagementModal.vue";
import { getRequestHeaders } from "@/utils";

interface ISwitchInterface {
    alias: string;
    id: number;
    type: string;
    status: {type: String};
}

export default defineComponent({
    name: "SwitchList",
    components: { TimerManagementModal },
    setup() {

        const switches = reactive<ISwitchInterface[]>([
        ]);

        const modals = reactive({
            edit: false,
            delete: false,
            timer: false,
        });

        const activeMenu = ref<number | null>(null);
        const editSwitch = reactive<ISwitchInterface>({ alias: "", id: 0, type: "", status: {type: ""} });
        let deleteSwitchId = ref<number | null>(null);

        const openMenu = (id: number) => (activeMenu.value = activeMenu.value === id ? null : id);

        const openEditModal = (switchItem: ISwitchInterface) => {
            Object.assign(editSwitch, switchItem);
            modals.edit = true;
        };

        const openDeleteModal = (id: number) => {
            deleteSwitchId.value = id;
            modals.delete = true;
        };

        const openTimerModal = (switchItem: ISwitchInterface) => {
            Object.assign(editSwitch, switchItem);
            modals.timer = true;
        };

        const closeModal = (type: keyof typeof modals) => (modals[type] = false);

        const saveEdit = () => {
            const switchItem = switches.find((item) => item.id === editSwitch.id);
            if (switchItem) switchItem.alias = editSwitch.alias;
            closeModal("edit");
        };

        const deleteSwitch = () => {
            if (deleteSwitchId.value !== null) {
                const index = switches.findIndex((item) => item.id === deleteSwitchId.value);
                if (index !== -1) switches.splice(index, 1);
                closeModal("delete");
            }
        };

        const API_URL = import.meta.env.VITE_BACKEND_URL;
        fetch(`${API_URL}/api/switches`, {
            headers: getRequestHeaders()
        }).then(x => x.json().then((res: ISwitchInterface[]) => {
            switches.splice(0, switches.length);
            res.forEach(x => switches.push(x));
        }));

        return {
            switches,
            modals,
            activeMenu,
            editSwitch,
            deleteSwitchId,
            openMenu,
            openEditModal,
            openDeleteModal,
            openTimerModal,
            closeModal,
            saveEdit,
            deleteSwitch,
            toggleSwitch: (id: number) => {
                const switchItem = switches.find((item) => item.id === id);
                if (switchItem) {
                    const API_URL = import.meta.env.VITE_BACKEND_URL;
                    const URI = switchItem.status.type === "On" ? "turn_off" : "turn_on";
                    fetch(`${API_URL}/api/${URI}/${switchItem.id}`, {
                        headers: getRequestHeaders()
                    }).then(x => x.json())
                    .then((x: {success: boolean}) => {
                        if (x.success)
                        {
                            switchItem.status.type = switchItem.status.type === "On" ? "Off" : "On";
                        }
                    });
                }
            },
        };
    },
});
</script>

<style scoped>
body {
    margin: 0;
    font-family: Arial, sans-serif;
}
</style>