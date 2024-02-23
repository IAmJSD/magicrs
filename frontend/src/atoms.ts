import { atom } from "jotai";

// The atom to define the active uploader ID.
export const uploaderIdAtom = atom(null as string | null);

// The atom to define the active hotkey.
export const hotkeyAtom = atom(null as string | null);
