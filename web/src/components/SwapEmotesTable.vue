<template>
  <div class="m-2 mt-0 border-red border-2 rounded-lg overflow-hidden">
    <table class="table-fixed w-full">
      <colgroup>
        <col class="w-[10%]" />
        <col />
        <col />
        <col />
        <col class="w-[10%]" />
      </colgroup>
      <thead>
        <tr v-for="headerGroup in table.getHeaderGroups()" :key="headerGroup.id" class="border-b border-red">
          <th
            v-for="header in headerGroup.headers"
            :key="header.id"
            :colSpan="header.colSpan"
            :class="[
              'border-r last:border-r-0 border-gray-500 px-2 py-4 font-bold header-item',
              header.column.getCanSort() ? 'cursor-pointer select-none' : '',
            ]"
            @click="header.column.getToggleSortingHandler()?.($event)"
          >
            <div class="flex justify-center items-center">
              <div v-if="header.column.getCanSort()" class="w-[24px] mr-4" />
              <FlexRender
                v-if="!header.isPlaceholder"
                :render="header.column.columnDef.header"
                :props="header.getContext()"
              />
              <SortDirection
                :class="['ml-4', header.column.getIsSorted() ? '' : 'transition-opacity opacity-0 dir-unsorted']"
                v-if="header.column.getCanSort()"
                :dir="header.column.getIsSorted()"
              />
            </div>
          </th>
        </tr>
      </thead>
      <tbody class="border-b border-gray-500">
        <tr
          v-for="row in table.getRowModel().rows"
          :key="row.id"
          class="hover:bg-gray-400 transition-colors odd:bg-gray-300 font-mono text-sm"
        >
          <td
            v-for="(cell, cellIdx) in row.getVisibleCells()"
            :key="cell.id"
            :class="['text-ellipsis overflow-hidden', cellIdx === 0 ? '' : 'px-4 py2']"
          >
            <FlexRender :render="cell.column.columnDef.cell" :props="cell.getContext()" />
          </td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<script setup lang="ts">
import {
  FlexRender,
  getCoreRowModel,
  useVueTable,
  ColumnDef,
  SortingState,
  getSortedRowModel,
} from '@tanstack/vue-table';
import { computed, ref, h } from 'vue';
import { SlotPlatform, SwapEmote } from '../api/types';
import EmoteImage from './EmoteImage.vue';
import SortDirection from './icons/SortDirection.vue';
import UntrackButton from './UntrackButton.vue';

const props = defineProps<{ items: SwapEmote[]; broadcasterId: string; rewardId: string }>();
const emit = defineEmits<{ untrack: [number] }>();
const dateTimeFormat = new Intl.DateTimeFormat(undefined, {
  year: '2-digit',
  month: 'short',
  day: '2-digit',
  hour: '2-digit',
  minute: '2-digit',
});

const columns = computed<ColumnDef<SwapEmote>[]>(() => [
  {
    accessorFn: row => {
      return [row.platform, row.emote_id] as const;
    },
    header: 'Image',
    cell: info => {
      const [platform, id] = info.getValue() as [SlotPlatform, string];
      return h(EmoteImage, { platform, id });
    },
    enableSorting: false,
  },
  {
    accessorKey: 'name',
    header: 'Name',
  },
  {
    accessorKey: 'added_by',
    header: 'Added By',
  },
  {
    accessorKey: 'added_at',
    header: 'Added At',
    cell: info => {
      const val = info.getValue() as string;
      return dateTimeFormat.format(Date.parse(val));
    },
    id: 'date',
  },
  {
    accessorFn: row => row.id,
    header: 'Untrack',
    cell: info =>
      h(UntrackButton, {
        emoteId: info.getValue() as number,
        onUntrack: id => emit('untrack', id),
      }),
    enableSorting: false,
  },
]);
const sorting = ref<SortingState>([{ desc: true, id: 'date' }]);

const table = useVueTable({
  get data() {
    return props.items;
  },
  get columns() {
    return columns.value;
  },
  state: {
    get sorting() {
      return sorting.value;
    },
  },
  onSortingChange: updaterOrValue => {
    sorting.value = typeof updaterOrValue === 'function' ? updaterOrValue(sorting.value) : updaterOrValue;
  },
  getCoreRowModel: getCoreRowModel(),
  getSortedRowModel: getSortedRowModel(),
});
</script>

<style scoped>
.header-item:hover .dir-unsorted {
  opacity: 90%;
}
</style>
