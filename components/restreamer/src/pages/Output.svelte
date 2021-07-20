<script lang="js">
  import Output from '../Output.svelte';
  import { onMount } from "svelte";
  import { Output as OutputSub } from '../api/graphql/client.graphql';
  import { subscribe } from 'svelte-apollo';

  export let state;
  export let params = {};

  const outputSub = subscribe(OutputSub, {
    errorPolicy: 'all',
    variables: {
      outputId: params.output_id,
      restreamId: params.restream_id,
    },
  });
</script>

<template>
  <div>
    {#if $outputSub.loading}
      Loading...
    {:else}
      {console.log($outputSub.data)}
    {/if}
  </div>

  {#each $state.data.allRestreams as restream}
    {#if restream.id === params.restream_id}
      {#each restream.outputs as output}
        {#if output.id === params.output_id}
          <section class="uk-section uk-section-muted single-output">
            <Output restream_id={restream.id} value={output} />
          </section>
        {/if}
      {/each}
    {/if}
  {/each}
</template>

<style lang="stylus">
  .single-output
    margin-top: 20px
    padding: 10px 20px 20px 20px
    max-width: 960px

    :global(.volume input)
        width: 90% !important
</style>
