<svelte:options immutable={true} />

<script lang="js">
  import { mutation, getClient, subscribe } from 'svelte-apollo';

  import {
    RemoveRestream,
    DisableAllOutputs,
    EnableAllOutputs,
    ExportRestream,
    Info,
  } from './api/graphql/client.graphql';

  import { showError } from './util';

  import { restreamModal, outputModal, exportModal } from './stores';

  import Confirm from './Confirm.svelte';
  import Input from './Input.svelte';
  import Output from './Output.svelte';
  import Toggle from './Toggle.svelte';

  const removeRestreamMutation = mutation(RemoveRestream);
  const disableAllOutputsMutation = mutation(DisableAllOutputs);
  const enableAllOutputsMutation = mutation(EnableAllOutputs);

  const gqlClient = getClient();
  const info = subscribe(Info, { errorPolicy: 'all' });

  export let public_host = 'localhost';
  export let value;

  $: deleteConfirmation = $info.data
    ? $info.data.info.deleteConfirmation
    : true;

  $: enableConfirmation = $info.data
    ? $info.data.info.enableConfirmation
    : true;

  $: allEnabled = value.outputs.every((o) => o.enabled);
  $: toggleStatusText = allEnabled ? 'Disable' : 'Enable';

  $: onlineCount = value.outputs.filter((o) => o.status === 'ONLINE').length;
  $: initCount = value.outputs.filter((o) => o.status === 'INITIALIZING')
    .length;
  $: offlineCount = value.outputs.filter((o) => o.status === 'OFFLINE').length;
  $: presentBitmask =
    (onlineCount > 0 ? 1 : 0) +
    2 * (initCount > 0 ? 1 : 0) +
    4 * (offlineCount > 0 ? 1 : 0);

  let enabledBitmask = 0;
  $: if (enabledBitmask === presentBitmask) {
    enabledBitmask = 0;
  }

  $: showAll =
    (enabledBitmask & presentBitmask) === presentBitmask ||
    (enabledBitmask & presentBitmask) === 0;
  $: showFiltered = {
    ONLINE: !showAll && (enabledBitmask & 1) === 1,
    INITIALIZING: !showAll && (enabledBitmask & 2) === 2,
    OFFLINE: !showAll && (enabledBitmask & 4) === 4,
  };

  function openEditRestreamModal() {
    const with_hls = value.input.endpoints.some((e) => e.kind === 'HLS');

    let pull_url = null;
    let backup = null;

    if (!!value.input.src && value.input.src.__typename === 'RemoteInputSrc') {
      pull_url = value.input.src.url;
    }

    if (
      !!value.input.src &&
      value.input.src.__typename === 'FailoverInputSrc'
    ) {
      backup = true;
      if (!!value.input.src.inputs[0].src) {
        pull_url = value.input.src.inputs[0].src.url;
      }
      if (!!value.input.src.inputs[1].src) {
        backup = value.input.src.inputs[1].src.url;
      }
    }

    restreamModal.openEdit(
      value.id,
      value.key,
      value.label,
      pull_url,
      backup,
      with_hls
    );
  }

  async function removeRestream() {
    try {
      await removeRestreamMutation({ variables: { id: value.id } });
    } catch (e) {
      showError(e.message);
    }
  }

  function openAddOutputModal() {
    outputModal.openAdd(value.id);
  }

  async function toggleAllOutputs() {
    if (value.outputs.length < 1) return;
    const variables = { restream_id: value.id };
    try {
      if (allEnabled) {
        await disableAllOutputsMutation({ variables });
      } else {
        await enableAllOutputsMutation({ variables });
      }
    } catch (e) {
      showError(e.message);
    }
  }

  async function openExportModal() {
    let resp;
    try {
      resp = await gqlClient.query({
        query: ExportRestream,
        variables: { id: value.id },
        fetchPolicy: 'no-cache',
      });
    } catch (e) {
      showError(e.message);
      return;
    }

    if (!!resp.data && !!resp.data.export) {
      exportModal.open(
        value.id,
        JSON.stringify(JSON.parse(resp.data.export), null, 2)
      );
    }
  }
</script>

<template>
  <div class="uk-section uk-section-muted uk-section-xsmall">
    <div class="left-buttons-area" />
    <div class="right-buttons-area" />
    <Confirm let:confirm>
      <button
        type="button"
        class="uk-close"
        uk-close
        on:click={deleteConfirmation
          ? () => confirm(removeRestream)
          : removeRestream}
      />
      <span slot="title"
        >Removing <code>{value.key}</code> input source for re-streaming</span
      >
      <span slot="description"
        >All its outputs will be removed too. You won't be able to undone this.</span
      >
      <span slot="confirm">Remove</span>
    </Confirm>

    <button
      class="uk-button uk-button-primary uk-button-small"
      on:click={openAddOutputModal}
    >
      <i class="fas fa-plus" />&nbsp;<span>Output</span>
    </button>

    <a
      class="export-import"
      href="/"
      on:click|preventDefault={openExportModal}
      title="Export/Import"
    >
      <i class="fas fa-share-square" />
    </a>

    {#if !!value.label}
      <span class="label">{value.label}</span>
    {/if}

    {#if value.outputs && value.outputs.length > 0}
      <span class="total">
        <span class="total-item-wrapper">
          {#if offlineCount > 0}
            <a
              href="/"
              on:click|preventDefault={() => (enabledBitmask ^= 4)}
              class:enabled={showFiltered['OFFLINE']}
              class="count uk-alert-danger">{offlineCount}</a
            >
          {/if}
        </span>
        <span class="total-item-wrapper">
          {#if initCount > 0}
            <a
              href="/"
              on:click|preventDefault={() => (enabledBitmask ^= 2)}
              class:enabled={showFiltered['INITIALIZING']}
              class="count uk-alert-warning">{initCount}</a
            >
          {/if}
        </span>
        <span class="total-item-wrapper">
          {#if onlineCount > 0}
            <a
              href="/"
              on:click|preventDefault={() => (enabledBitmask ^= 1)}
              class:enabled={showFiltered['ONLINE']}
              class="count uk-alert-success">{onlineCount}</a
            >
          {/if}
        </span>

        <Confirm let:confirm>
          <Toggle
            id="all-outputs-toggle-{value.id}"
            checked={allEnabled}
            title="{toggleStatusText} all outputs"
            confirmFn={enableConfirmation ? confirm : undefined}
            onChangeFn={toggleAllOutputs}
          />
          <span slot="title"
            >{toggleStatusText} all outputs of <code>{value.key}</code> input</span
          >
          <span slot="description">Are you sure about it?</span>
          <span slot="confirm">{toggleStatusText}</span>
        </Confirm>
      </span>
    {/if}

    <a
      class="edit-input"
      href="/"
      on:click|preventDefault={openEditRestreamModal}
    >
      <i class="far fa-edit" title="Edit input" />
    </a>
    <Input
      {public_host}
      restream_id={value.id}
      restream_key={value.key}
      value={value.input}
    />
    {#if !!value.input.src && value.input.src.__typename === 'FailoverInputSrc'}
      {#each value.input.src.inputs as input}
        <Input
          {public_host}
          restream_id={value.id}
          restream_key={value.key}
          value={input}
        />
      {/each}
    {/if}

    {#if value.outputs && value.outputs.length > 0}
      <div class="uk-grid uk-grid-small" uk-grid>
        {#each value.outputs as output}
          <Output
            {public_host}
            restream_id={value.id}
            value={output}
            hidden={!showAll && !showFiltered[output.status]}
          />
        {/each}
      </div>
    {/if}
  </div>
</template>

<style lang="stylus">
  .uk-section
    position: relative
    margin-top: 20px
    padding-left: 10px
    padding-right: @padding-left

    &:hover
      .uk-close, .uk-button-small
      .edit-input, .export-import
        opacity: 1

    .uk-button-small
      float: right
      font-size: 0.7rem
      margin-top: -2px
      opacity: 0
      transition: opacity .3s ease
      &:hover
        opacity: 1

    .total
      float: right
      margin-right: 20px
      .total-item-wrapper
        min-width: 32px
        display: inline-flex
        .count
          width: 100%
          text-align: center
          margin-right: 2px
          background-color: inherit
          padding: 1px 4px
          border-radius: 2px
          outline: none
          &:hover, &.enabled
            background-color: #cecece
          &:hover
            color: inherit
            text-decoration: none

    .edit-input, .export-import, .uk-close
      position: absolute
      opacity: 0
      transition: opacity .3s ease
      &:hover
        opacity: 1
    .edit-input, .export-import
      color: #666
      outline: none
      &:hover
        text-decoration: none
        color: #444
    .edit-input
      left: -25px
    .export-import
      right: -25px
    .uk-close
      right: -21px
      top: -15px

    .left-buttons-area, .right-buttons-area
      position: absolute
      width: 34px
    .left-buttons-area
      right: 100%
      top: 0
      height: 100%
    .right-buttons-area
      left: 100%
      top: -20px
      height: calc(20px + 100%)

    .uk-grid
      margin-top: 10px
      margin-left: -10px

    .label
      position: absolute
      top: -12px
      left: 0
      padding: 2px 10px
      border-top-left-radius: 4px
      border-top-right-radius: 4px
      background-color: #f8f8f8
</style>
