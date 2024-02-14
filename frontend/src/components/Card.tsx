import { PropsWithChildren } from "react";

export type CardProps = {
  title: string;
  icon: React.ForwardRefExoticComponent<
    Omit<React.SVGProps<SVGSVGElement>, "ref">
  >;
};

export function Card(props: PropsWithChildren<CardProps>) {
  return (
    <div className="flex flex-col w-full h-full p-6 transition bg-surface target:ring focus:outline-none focus:ring svelte-o31m6n hoverable size-lg bg-rosePine-surface rounded-3xl">
      <div className="flex items-center justify-center h-14 w-14 shrink-0 rounded-2xl bg-gradient-to-br from-rosePine-foam to-rosePine-pine text-surface">
        <props.icon className="w-6 stroke-2 stroke-rosePine-base" />
      </div>
      <div className="h-6" />
      <p className="text-lg font-bold leading-none tracking-tight">
        {props.title}
      </p>
      <div className="h-6" />
      <span>{props.children}</span>
    </div>
  );
}
