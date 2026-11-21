export function LensHeader({
  title,
  description,
}: {
  title: string;
  description: string;
}) {
  return (
    <header className="border-line border-b px-5 pt-[18px] pb-3.5">
      <h1 className="font-pixel text-[30px] leading-none tracking-[0.005em]">
        {title}
      </h1>
      <p className="mt-1.5 max-w-[64ch] text-[12.5px] text-ink-2">
        {description}
      </p>
    </header>
  );
}
