const countFormat = new Intl.NumberFormat("en-US");

function count(value: number): string {
  return countFormat.format(value);
}

export const english = {
  start: {
    title: "No board running",
    instruction: "Start one in a project's checkout with",
    afterwards: "Blossom opens it as soon as it starts.",
  },
  status: {
    handle: (handle: string) => `@${handle}`,
    copying: (stored: number, total: number | null) => {
      if (total === null) {
        return `Copying ${count(stored)}`;
      }
      return `Copying ${count(stored)} of ${count(total)}`;
    },
    copied: (stored: number) => {
      if (stored === 1) {
        return "1 message";
      }
      return `${count(stored)} messages`;
    },
  },
};
