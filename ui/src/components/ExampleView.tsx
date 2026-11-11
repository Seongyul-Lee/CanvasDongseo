// 예시 컴포넌트(components): ViewModel 조각을 받은 그대로 그린다. 무엇을 보여 줄지는 ViewModel이 정한다.
// 첫 기능이 들어오면 지운다(design/samples/example/README.md)
import { CircleAlert, CircleCheck, Info, TriangleAlert } from "lucide-react";
import type { ExampleBanner, ExampleProgressEvent, ExampleViewModel } from "../viewmodels/example.ts";
import { Alert, AlertDescription, AlertTitle } from "./ui/alert.tsx";
import { Badge } from "./ui/badge.tsx";

const BANNER_ICON = {
  info: Info,
  success: CircleCheck,
  warning: TriangleAlert,
  error: CircleAlert,
} satisfies Record<ExampleBanner["kind"], unknown>;

export function ExampleView({
  viewModel,
  progress,
}: {
  viewModel: ExampleViewModel;
  progress: ExampleProgressEvent | null;
}) {
  const BannerIcon = viewModel.banner && BANNER_ICON[viewModel.banner.kind];
  return (
    <main className="mx-auto flex h-full max-w-xl flex-col gap-4 overflow-auto p-6 text-sm text-foreground">
      <header className="flex flex-col gap-1">
        <h1 className="text-lg font-semibold">{viewModel.title}</h1>
        <p className="text-muted-foreground">{viewModel.description}</p>
      </header>

      {viewModel.banner && BannerIcon && (
        <Alert variant={viewModel.banner.kind}>
          <BannerIcon />
          <AlertTitle>{viewModel.banner.title}</AlertTitle>
          <AlertDescription>{viewModel.banner.message}</AlertDescription>
        </Alert>
      )}

      {progress && (
        <section className="flex flex-col gap-1.5" aria-live="polite">
          <div className="flex items-center justify-between text-xs">
            <span>{progress.label}</span>
            <span className="text-muted-foreground tabular-nums">{progress.percent}%</span>
          </div>
          <div
            className="h-1.5 overflow-hidden rounded-full bg-muted"
            role="progressbar"
            aria-valuemin={0}
            aria-valuemax={100}
            aria-valuenow={progress.percent}
          >
            <div
              className="h-full rounded-full bg-primary transition-[width] duration-300"
              style={{ width: `${progress.percent}%` }}
            />
          </div>
        </section>
      )}

      <ul className="flex flex-col divide-y rounded-lg border empty:hidden">
        {viewModel.items.map((item) => (
          <li key={item.id} className="flex items-center justify-between gap-3 px-3 py-2">
            <div className="flex min-w-0 flex-col">
              <span className="truncate font-medium">{item.name}</span>
              <span className="truncate text-xs text-muted-foreground">{item.detail}</span>
            </div>
            <Badge variant={item.badge.kind}>{item.badge.label}</Badge>
          </li>
        ))}
      </ul>

      {viewModel.emptyMessage && <p className="text-center text-muted-foreground">{viewModel.emptyMessage}</p>}
    </main>
  );
}
