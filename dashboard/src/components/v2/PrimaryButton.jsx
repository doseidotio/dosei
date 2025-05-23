import React from "react";
import Link from "next/link";
import clsx from "clsx";

export const PrimaryButton = ({ icon, title = "", href = "", onClick = null, ...props }) => {
  const className = clsx(
    "inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md",
    // light theme
    "text-blue-700 bg-blue-100 hover:bg-blue-200",
    // dark theme
    "dark:text-black dark:bg-white dark:hover:bg-white/80 dark:ring-white/20"
  );
  if (onClick) {
    return (
      <button
        onClick={onClick}
        className={className}
        {...props}
      >
        { icon && React.createElement(icon, { className: "-ml-1 mr-2 h-5 w-5", "aria-hidden": "true" }) }
        <span>{title}</span>
      </button>
    );
  }
  return (
    <Link
      className={className}
      href={href}
      {...props}
    >
      { icon && React.createElement(icon, { className: "-ml-1 mr-2 h-5 w-5", "aria-hidden": "true" }) }
      <span>{title}</span>
    </Link>
  )
};
