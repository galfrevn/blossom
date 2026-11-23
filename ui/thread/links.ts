import { useEffect } from "react";

const MIDDLE_BUTTON = 1;

type LinkHolder = React.RefObject<HTMLElement | null>;

function linkClicked(event: MouseEvent): string | undefined {
  if (!(event.target instanceof Element)) {
    return undefined;
  }
  const anchor = event.target.closest("a[href]");
  return anchor?.getAttribute("href") ?? undefined;
}

export function useOpeningLinksInTheBrowser(
  holder: LinkHolder,
  openLink: (link: string) => Promise<void>,
) {
  useEffect(() => {
    const element = holder.current;
    if (element === null) {
      return;
    }
    const openTheClickedLink = (event: MouseEvent) => {
      const link = linkClicked(event);
      if (link === undefined) {
        return;
      }
      event.preventDefault();
      openLink(link).catch((error: unknown) => {
        console.error("blossom: a link could not be opened", error);
      });
    };
    const openTheLinkClickedWithTheMiddleButton = (event: MouseEvent) => {
      if (event.button !== MIDDLE_BUTTON) {
        return;
      }
      openTheClickedLink(event);
    };
    element.addEventListener("click", openTheClickedLink);
    element.addEventListener("auxclick", openTheLinkClickedWithTheMiddleButton);
    return () => {
      element.removeEventListener("click", openTheClickedLink);
      element.removeEventListener(
        "auxclick",
        openTheLinkClickedWithTheMiddleButton,
      );
    };
  }, [holder, openLink]);
}
