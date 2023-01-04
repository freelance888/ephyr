<script lang="js">
  import { getClient, mutation } from 'svelte-apollo';

  import { DvrFiles, RemoveDvrFile } from '../../api/client.graphql';

  import { showError } from '../utils/util';

  const gqlClient = getClient();

  const removeDvrFileMutation = mutation(RemoveDvrFile);

  export let public_host;
  export let id;

  let files = [];

  async function open() {
    let resp;
    try {
      resp = await gqlClient.query({
        query: DvrFiles,
        variables: { id },
        fetchPolicy: 'no-cache',
      });
    } catch (e) {
      showError(e.message);
      return;
    }

    files =
      !!resp.data && !!resp.data.dvrFiles && resp.data.dvrFiles.length > 0
        ? resp.data.dvrFiles
        : [];
  }

  async function remove(path) {
    const variables = { path };
    try {
      await removeDvrFileMutation({ variables });
    } catch (e) {
      showError(e.message);
      return;
    }

    await open();
  }
</script>

<template>
  <slot {open} />

  {#if files.length > 0}
    <div class="uk-modal uk-open">
      <div class="uk-modal-dialog uk-modal-body">
        <h2 class="uk-modal-title">Recorded files</h2>
        <button
          class="uk-modal-close-outside"
          uk-close
          type="button"
          on:click={() => (files = [])}
        />

        {#each files as file}
          <div class="record">
            <a
              download
              target="_blank"
              rel="noopener noreferrer"
              title="Download recorded file"
              href="http://{public_host}:8000/dvr/{file}"
              >{file.split('/').slice(-1)[0]}</a
            >
            <button
              uk-close
              type="button"
              title="Remove recorded file"
              on:click={() => remove(file)}
            />
          </div>
        {/each}
      </div>
    </div>
  {/if}
</template>

<style lang="stylus">
  .uk-modal
    &.uk-open
      display: block

    .uk-modal-title
      font-size: 1.5rem

  .record
    a
      color: var(--primary-text-color)

    button
      margin-left: 15px
      margin-top: 3px
      float: right
    &:not(:hover)
      button
        opacity: 0

    & + .record
      margin-top: 10px
</style>
