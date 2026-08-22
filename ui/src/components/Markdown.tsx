import Markdown from "react-markdown";

export function MarkdownContent({ children, className = "" }: { children: string; className?: string }) {
  return (
    <div className={`markdown ${className}`.trim()}>
      <Markdown skipHtml>{children}</Markdown>
    </div>
  );
}
