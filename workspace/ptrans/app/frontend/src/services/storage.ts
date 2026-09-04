export interface DepositNote {
  id: string;
  nullifierSecret: string; // hex
  secret: string; // hex
  amount: number; // lamports
  commitment: string; // hex
  nullifierHash: string; // hex
  timestamp: number;
  used: boolean;
}

const STORAGE_KEY = "ptrans_deposits";

/**
 * Получить все записки из localStorage
 */
export function getDepositNotes(): DepositNote[] {
  try {
    const data = localStorage.getItem(STORAGE_KEY);
    if (!data) return [];
    return JSON.parse(data) as DepositNote[];
  } catch {
    return [];
  }
}

/**
 * Сохранить новую записку
 */
export function saveDepositNote(note: DepositNote): DepositNote {
  const notes = getDepositNotes();
  notes.push(note);
  localStorage.setItem(STORAGE_KEY, JSON.stringify(notes));
  return note;
}

/**
 * Получить записку по nullifierHash
 */
export function getDepositNoteByNullifier(nullifierHash: string): DepositNote | null {
  const notes = getDepositNotes();
  return notes.find((n) => n.nullifierHash === nullifierHash) || null;
}

/**
 * Получить записку по id
 */
export function getDepositNoteById(id: string): DepositNote | null {
  const notes = getDepositNotes();
  return notes.find((n) => n.id === id) || null;
}

/**
 * Отметить записку как использованную
 */
export function markNoteAsUsed(nullifierHash: string): DepositNote | null {
  const notes = getDepositNotes();
  const index = notes.findIndex((n) => n.nullifierHash === nullifierHash);

  if (index === -1) return null;

  const existingNote = notes[index];
  if (!existingNote) return null;

  const updatedNote: DepositNote = {
    id: existingNote.id,
    nullifierSecret: existingNote.nullifierSecret,
    secret: existingNote.secret,
    amount: existingNote.amount,
    commitment: existingNote.commitment,
    nullifierHash: existingNote.nullifierHash,
    timestamp: existingNote.timestamp,
    used: true,
  };

  notes[index] = updatedNote;
  localStorage.setItem(STORAGE_KEY, JSON.stringify(notes));
  return updatedNote;
}

/**
 * Удалить записку
 */
export function removeDepositNote(nullifierHash: string): boolean {
  const notes = getDepositNotes();
  const filtered = notes.filter((n) => n.nullifierHash !== nullifierHash);

  if (filtered.length === notes.length) return false;

  localStorage.setItem(STORAGE_KEY, JSON.stringify(filtered));
  return true;
}

/**
 * Получить только неиспользованные записки
 */
export function getUnusedDepositNotes(): DepositNote[] {
  return getDepositNotes().filter((n) => !n.used);
}

/**
 * Очистить все записки
 */
export function clearDepositNotes(): void {
  localStorage.removeItem(STORAGE_KEY);
}
