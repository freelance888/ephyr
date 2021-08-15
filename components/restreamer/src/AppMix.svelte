<script lang="ts">
  import { createGraphQlClient } from './util';
  import { setClient, subscribe } from "svelte-apollo";
  import Shell from "./Shell.svelte";
  import Output from './Output.svelte';
  import { Output as Mix, Info } from './api/graphql/mix.graphql';
  import { onDestroy } from 'svelte';

  const gqlClient = createGraphQlClient('/api-mix', () => isOnline = true, () => isOnline = false);
  setClient(gqlClient);

  let isOnline = false;

  const urlParams = new URLSearchParams(window.location.search);
  const output_id = urlParams.get('output');
  const restream_id = urlParams.get('id');

  const info = subscribe(Info, {errorPolicy: 'all'});
  const mix = subscribe(Mix, {
    errorPolicy: 'all',
    variables: {
      outputId: output_id,
      restreamId: restream_id,
    },
  });

  $: mixError = $mix && $mix.error;
  $: infoError = $info && $info.error;
  $: isStateLoading = !isOnline || $mix.loading;
  $: canRenderMainComponent = isOnline && $mix.data;
  $: error = $mixError || $infoError;

  onDestroy(
      info.subscribe((i) => {
        if (!i.loading && i.data) {
          const title = i.data.title;
          document.title = title || 'Ephyr re-streamer';
        }
      })
  );

</script>

<template>
  <Shell {canRenderMainComponent} {isStateLoading} {error}>
    <section slot="main" class="uk-section uk-section-muted single-output">
      <Output restream_id={restream_id} value={$mix.data.output}  />
    </section>
  </Shell>

</template>

<style lang="stylus">
  .single-output
    margin-top: 20px
    padding: 10px 20px 20px 20px
    max-width: 960px

    :global(.volume input)
      width: 90% !important
</style>
