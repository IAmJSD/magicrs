export default ({ thin }: { thin?: boolean }) =>
    <hr className={`${thin ? "my-3" : "my-5"} dark:border-neutral-700 border-neutral-200`} />;
