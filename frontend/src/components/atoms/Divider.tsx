export default ({ thin }: { thin?: boolean }) =>
    <hr className={`my-${thin ? "3" : "5"} dark:border-neutral-700 border-neutral-200`} />;
