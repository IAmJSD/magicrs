import { useCallback, useEffect, useId, useState } from "react";
import { Props } from "./shared";

const VALID_SUBDOMAIN_REGEX = /^[a-z0-9-]+$/;

function SubdomainInput({ value, setValue }: {
    value: string | null;
    setValue: (value: string | null) => void;
}) {
    // Defines the error state.
    const [error, setError] = useState<string | null>(null);

    // Defines the callback for the input event.
    const inputCb = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
        const val = e.target.value;
        if (val === "") {
            setValue(null);
            setError(null);
            return;
        }

        if (!VALID_SUBDOMAIN_REGEX.test(val)) {
            setError("Subdomains can only contain lowercase letters, numbers, and hyphens.");
            return;
        }

        setValue(val);
        setError(null);
    }, [setValue]);

    // Return the input field and error message.
    return <>
        <div className="flex-col">
            <input
                type="text"
                value={value || ""}
                onChange={inputCb}
                placeholder="Sub-domain"
                className="p-1 rounded text-black"
            />
            {error ? <p className="text-red-500">{error}</p> : null}
        </div>
        <div className="flex-col mx-2">.</div>
    </>
}

function DomainSelector({ initValue, domainsResult, setConfig }: {
    initValue: [string | null, string, boolean] | null;
    domainsResult: {[key: string]: any} | Error | null;
    setConfig: (value: [string | null, string, boolean]) => Promise<void>;
}) {
    // Create the state for the value.
    const [value, setValue] = useState(initValue);

    // Defines the callback for sub-domain handling.
    const subdomainCb = useCallback((v: string | null) => {
        setValue([v, value![1], true]);
        setConfig([v, value![1], true]);
    }, [value]);

    // Defines the callback for the domain select event.
    const domainSelectCb = useCallback((e: React.ChangeEvent<HTMLSelectElement>) => {
        // Handle the value that was selected.
        const domainId = e.target.value;
        const allowSubdomain = e.target.selectedOptions[0].dataset.allowsSubdomain === "true";

        // Build the new value.
        const subdomain = (value || [null])[0];
        setValue([subdomain, domainId, allowSubdomain]);
        setConfig([subdomain, domainId, allowSubdomain]);
    }, [value]);

    // Handle if the domain result is still loading.
    if (domainsResult === null) return <p className="mt-2">
        <i>Loading domain information from elixi.re...</i>
    </p>;

    // Handle if the domain result is an error.
    if (domainsResult instanceof Error) return <p className="mt-2">
        Error loading domain information from elixi.re:<code className="ml-1">{domainsResult.message}</code>
    </p>;

    // The domains object is a map of numbers -> domain names. Sort them by the number which is the key.
    const domains = Object.entries(domainsResult.domains).sort((a, b) => a[0].localeCompare(b[0])) as [string, string][];

    // Return a form with flexbox to show the domain selector.
    return <form onSubmit={e => e.preventDefault()} className="flex mt-2 align-middle items-center">
        {
            value?.[2] === true ? <SubdomainInput
                value={value[0]}
                setValue={subdomainCb}
            /> : null
        }
        <select
            onChange={domainSelectCb}
            defaultValue={value?.[1] || "Select a domain..."}
            className="flex-col w-40 dark:text-black"
        >
            <option value="Select a domain..." disabled>Select a domain...</option>
            {domains.map(([id, domain]) => {
                // Check if the domain starts with *., if so, trim it but store it in data.
                let allowsSubdomain = false;
                if (domain.startsWith("*.")) {
                    domain = domain.slice(2);
                    allowsSubdomain = true;
                }

                // Return the option.
                return <option key={id} value={id} data-allows-subdomain={allowsSubdomain}>
                    {domain}
                </option>;
            })}
        </select>
    </form>;
}

const NO_SELECTION = Symbol("NO_SELECTION");

export default function ElixireDomainConfig(props: Props) {
    const [value, setValue] = useState(props.value || NO_SELECTION);
    const [domainsResult, setDomainsResult] = useState<{[key: string]: any} | Error | null>(null);

    useEffect(() => {
        let cancelled = false;

        fetch("https://elixire-domains-list-proxy.astrids.workers.dev/").then(async res => {
            if (res.ok) {
                const data = await res.json();
                if (cancelled) return;
                setDomainsResult(data);
                return;
            }

            throw new Error(`Failed to fetch domains list: ${res.status} ${res.statusText}`);
        }).catch(e => {
            if (cancelled) return;
            setDomainsResult(e);
        });

        return () => { cancelled = true; };
    });
    const id1 = useId();
    const id2 = useId();

    return <>
        <form onSubmit={e => e.preventDefault()}>
            <label htmlFor={id1} className="flex items-center align-middle">
                <input
                    type="radio" className="mr-1"
                    checked={value === NO_SELECTION}
                    onChange={() => {
                        setValue(NO_SELECTION);
                        props.set(undefined);
                    }}
                    id={id1} name="domain_management"
                />

                Use elixi.re to manage my used domain
            </label>

            <label htmlFor={id2} className="flex items-center align-middle">
                <input
                    type="radio" className="mr-1"
                    checked={value !== NO_SELECTION}
                    onChange={() => setValue(null)}
                    id={id2} name="domain_management"
                />

                Use MagicCap to manage my used domain
            </label>
        </form>

        {value === NO_SELECTION ? null : <DomainSelector
            initValue={value} domainsResult={domainsResult} setConfig={props.set}
        />}
    </>;
}
