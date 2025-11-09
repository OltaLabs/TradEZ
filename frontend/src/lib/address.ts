import { ethers } from "ethers";

export const normalizeAddressLike = (value: unknown): string | null => {
  if (typeof value === "string") {
    return value.toLowerCase();
  }
  if (Array.isArray(value)) {
    if (value.length === 1 && typeof value[0] === "string") {
      return (value[0] as string).toLowerCase();
    }
    if (value.every((entry) => typeof entry === "number")) {
      try {
        return ethers.hexlify(Uint8Array.from(value as number[])).toLowerCase();
      } catch {
        return null;
      }
    }
  }
  return null;
};
