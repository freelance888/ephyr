<script lang="ts">
  import { Info, State } from './api/graphql/client.graphql';
  import { setClient, subscribe } from "svelte-apollo";
    import { createGraphQlClient } from './util';

  export let gqlClient = createGraphQlClient('/api', () => isOnline = true, () => isOnline = false);
  setClient(gqlClient);

  export let isOnline = false;
  export let info = subscribe(Info, { errorPolicy: 'all' });
  export let state = subscribe(State, { errorPolicy: 'all' });

</script>

<template>
    {#if isOnline && $info.data && $state.data }
        <slot></slot>
    {/if}
</template>
