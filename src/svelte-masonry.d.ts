declare module 'svelte-masonry' {
  import type { SvelteComponent } from 'svelte';

  interface MasonryProps {
    items?: any[];
    colWidth?: string;
    gridGap?: string;
    padding?: string;
    stretchFirst?: boolean;
    reset?: boolean;
    refreshLayout?: () => void;
    children?: any;
  }

  export default class Masonry extends SvelteComponent<MasonryProps> {}
}
