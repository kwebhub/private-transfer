import { defineStore } from "pinia";
import { ref, computed } from "vue";
import type { DepositNote } from "@/services/storage";
import {
  getDepositNotes,
  saveDepositNote,
  markNoteAsUsed,
  removeDepositNote,
  clearDepositNotes,
} from "@/services/storage";
import { bytesToHex } from "@/services/crypto";

export const useDepositsStore = defineStore("deposits", () => {
  // Состояние
  const notes = ref<DepositNote[]>([]);

  // Геттеры
  const unusedNotes = computed(() => notes.value.filter((n) => !n.used));
  const usedNotes = computed(() => notes.value.filter((n) => n.used));
  const totalCount = computed(() => notes.value.length);
  const unusedCount = computed(() => unusedNotes.value.length);

  // Действия
  function loadNotes() {
    notes.value = getDepositNotes();
  }

  function addNote(
    nullifierSecret: Uint8Array,
    secret: Uint8Array,
    amount: number,
    commitment: Uint8Array,
    nullifierHash: Uint8Array,
  ): DepositNote {
    const newNote: DepositNote = {
      id: crypto.randomUUID(),
      nullifierSecret: bytesToHex(nullifierSecret),
      secret: bytesToHex(secret),
      amount,
      commitment: bytesToHex(commitment),
      nullifierHash: bytesToHex(nullifierHash),
      timestamp: Date.now(),
      used: false,
    };

    const saved = saveDepositNote(newNote);
    notes.value.push(saved);
    return saved;
  }

  function markUsed(nullifierHash: string): boolean {
    const updated = markNoteAsUsed(nullifierHash);
    if (updated) {
      const index = notes.value.findIndex((n) => n.nullifierHash === nullifierHash);
      if (index !== -1) {
        notes.value[index] = updated;
      }
      return true;
    }
    return false;
  }

  function removeNote(nullifierHash: string): boolean {
    const result = removeDepositNote(nullifierHash);
    if (result) {
      notes.value = notes.value.filter((n) => n.nullifierHash !== nullifierHash);
    }
    return result;
  }

  function getNoteByNullifier(nullifierHash: string): DepositNote | null {
    return notes.value.find((n) => n.nullifierHash === nullifierHash) || null;
  }

  function clearAll() {
    notes.value = [];
    clearDepositNotes();
  }

  // Инициализация
  loadNotes();

  return {
    // Состояние
    notes,
    // Геттеры
    unusedNotes,
    usedNotes,
    totalCount,
    unusedCount,
    // Действия
    loadNotes,
    addNote,
    markUsed,
    removeNote,
    getNoteByNullifier,
    clearAll,
  };
});
