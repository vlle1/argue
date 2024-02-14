import { ArrowTopRightOnSquareIcon } from "@heroicons/react/24/outline";
import { FormEvent } from "react";

const onSubmit = (e: FormEvent<HTMLFormElement>) => {
  e.preventDefault();
  const formData = new FormData(e.currentTarget);
};

export function LandingPage() {
  const items = ["God is dead.", "The earth is flat."];
  return (
    <div className="inline-flex flex-col items-center justify-center bg-rosePine-base w-dvw h-dvh text-rosePine-text">
      <form
        className="flex flex-col gap-2 text-5xl font-semibold caret-rosePine-subtle"
        onSubmit={(e) => onSubmit(e)}
      >
        <span>
          <span className="font-bold text-rosePine-iris">Argue</span> whether,
        </span>
        <span className="flex flex-row">
          <input
            name="text"
            className="bg-transparent outline-none ftransition-all"
            type="search"
            placeholder={items[Math.floor(Math.random() * items.length)]}
            autoFocus
          />
          <ArrowTopRightOnSquareIcon className="stroke-rosePine-subtle" />
        </span>
      </form>
    </div>
  );
}
