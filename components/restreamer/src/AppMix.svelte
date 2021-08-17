<script lang="js">
  import { createGraphQlClient } from './util';
  import { setClient, subscribe } from 'svelte-apollo';
  import Shell from './Shell.svelte';
  import Output from './Output.svelte';
  import {
    Output as Mix,
    TuneVolume,
    TuneDelay,
  } from './api/graphql/mix.graphql';

  const mutations = { TuneVolume, TuneDelay };

  const gqlClient = createGraphQlClient(
    '/api-mix',
    () => (isOnline = true),
    () => (isOnline = false)
  );
  setClient(gqlClient);

  let isOnline = false;

  const urlParams = new URLSearchParams(window.location.search);
  const output_id = urlParams.get('output');
  const restream_id = urlParams.get('id');

  const mix = subscribe(Mix, {
    errorPolicy: 'all',
    variables: {
      outputId: output_id,
      restreamId: restream_id,
    },
  });

  $: error = $mix && $mix.error;
  $: isLoading = !isOnline || $mix.loading;
  $: canRenderMainComponent = isOnline && $mix.data;
</script>

<template>
  <Shell {canRenderMainComponent} {isLoading} {error}>
    <section slot="main" class="uk-section uk-section-muted single-output">
      <Output {restream_id} value={$mix.data.output} {mutations} />
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
