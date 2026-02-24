import { useState, useEffect } from 'react';

// Hook to responsively determine columns based on window width
export function useResponsiveColumns(): number {
    const [cols, setCols] = useState(getColumns(window.innerWidth));

    useEffect(() => {
        let t: ReturnType<typeof setTimeout>;
        const handler = () => {
            clearTimeout(t);
            t = setTimeout(() => setCols(getColumns(window.innerWidth)), 150);
        };

        window.addEventListener('resize', handler);
        return () => {
            window.removeEventListener('resize', handler);
            clearTimeout(t);
        };
    }, []);

    return cols;
}

function getColumns(w: number) {
    if (w < 600) return 2;
    if (w < 900) return 3;
    if (w < 1200) return 4;
    if (w < 1600) return 5;
    return 6;
}
