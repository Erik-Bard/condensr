export function LinkGlyph({ size }: { size: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none">
      <path
        d="M9 15 L15 9 M10.5 7 L13 5a4 4 0 0 1 6 6l-2 1.5 M13.5 17 L11 19a4 4 0 0 1-6-6l2-1.5"
        stroke="currentColor"
        strokeWidth="1.7"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}
