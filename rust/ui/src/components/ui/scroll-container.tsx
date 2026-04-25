interface ScrollContainerProps extends React.HTMLAttributes<HTMLDivElement> {
  children: React.ReactNode
}

export function ScrollContainer({ className = '', children, ...props }: ScrollContainerProps) {
  return (
    <div
      className={`overflow-y-auto overflow-x-hidden h-full ${className}`}
      {...props}
    >
      {children}
    </div>
  )
}