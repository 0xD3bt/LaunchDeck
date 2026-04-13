(function initLaunchDeckQuotePreviewDomain(global) {
  const PUMP_TOKEN_DECIMALS = 6;
  const BONK_TOKEN_DECIMALS = 6;
  const BAGS_TOKEN_DECIMALS = 9;
  const PUMP_TOTAL_SUPPLY_RAW = 1_000_000_000n * (10n ** 6n);
  const BAGS_TOTAL_SUPPLY_RAW = 1_000_000_000n * (10n ** 9n);
  const BONK_FEE_RATE_DENOMINATOR = 1_000_000n;
  const BAGS_FEE_DENOMINATOR = 1_000_000_000n;
  const BAGS_RESOLUTION_BITS = 64n;
  const BONK_Q64 = 1n << 64n;
  const BAGS_INITIAL_SQRT_PRICE = 3141367320245630n;
  const BAGS_CURVE_POINTS = [
    {
      sqrtPrice: 6401204812200420n,
      liquidity: 3929368168768468756200000000000000n,
    },
    {
      sqrtPrice: 13043817825332782n,
      liquidity: 2425988008058820449100000000000000n,
    },
  ];

  function create(config) {
    const state = config && config.state && typeof config.state === "object" ? config.state : {};
    const helpers = config && config.helpers && typeof config.helpers === "object" ? config.helpers : {};

    const getStartupWarmPayload = typeof state.getStartupWarmPayload === "function"
      ? state.getStartupWarmPayload
      : (() => null);
    const getStoredPreviewInputs = typeof state.getStoredPreviewInputs === "function"
      ? state.getStoredPreviewInputs
      : (() => null);
    const formatBigIntDecimal = typeof helpers.formatBigIntDecimal === "function"
      ? helpers.formatBigIntDecimal
      : defaultFormatBigIntDecimal;

    function computeLocalQuote(shape) {
      const normalizedShape = normalizeShape(shape);
      if (!normalizedShape.amount) return null;
      const previewInputs = getEffectivePreviewInputs();
      switch (normalizedShape.launchpad) {
        case "pump":
          return computePumpQuote(normalizedShape, previewInputs.pump, formatBigIntDecimal);
        case "bonk":
          return computeBonkQuote(normalizedShape, previewInputs.bonk, formatBigIntDecimal);
        case "bagsapp":
          return computeBagsQuote(normalizedShape, formatBigIntDecimal);
        default:
          return {
            quote: null,
            placeholder: "Preview unavailable for this launchpad.",
          };
      }
    }

    function extractPreviewInputsFromStartupWarm(payload) {
      return normalizePreviewInputs(buildPreviewInputsFromStartupWarm(payload));
    }

    function normalizePreviewInputsValue(value) {
      return normalizePreviewInputs(value);
    }

    function mergePreviewInputs(baseValue, updateValue) {
      const base = normalizePreviewInputs(baseValue);
      const update = normalizePreviewInputs(updateValue);
      return {
        schemaVersion: 1,
        pump: update.pump || base.pump || null,
        bonk: {
          defaultsByKey: {
            ...(base.bonk && base.bonk.defaultsByKey ? base.bonk.defaultsByKey : {}),
            ...(update.bonk && update.bonk.defaultsByKey ? update.bonk.defaultsByKey : {}),
          },
          usd1Approx: update.bonk && update.bonk.usd1Approx
            ? update.bonk.usd1Approx
            : (base.bonk && base.bonk.usd1Approx ? base.bonk.usd1Approx : null),
        },
      };
    }

    function captureBonkUsd1ApproxFromReport(report) {
      const entry = report && typeof report === "object" ? report : {};
      const launchpad = String(entry.launchpad || "").trim().toLowerCase();
      const summary = entry.bonkUsd1Launch && typeof entry.bonkUsd1Launch === "object"
        ? entry.bonkUsd1Launch
        : null;
      if (launchpad !== "bonk" || !summary) return null;
      const inputLamports = decimalStringToRaw(summary.inputSol, 9);
      const minQuoteOutRaw = decimalStringToRaw(summary.minQuoteOut, 6);
      const expectedQuoteOutRaw = decimalStringToRaw(summary.expectedQuoteOut, 6);
      const quoteOutAmount = minQuoteOutRaw || expectedQuoteOutRaw;
      if (!inputLamports || !quoteOutAmount) return null;
      return normalizePreviewInputs({
        bonk: {
          usd1Approx: {
            inputLamports,
            quoteOutAmount,
            source: minQuoteOutRaw ? "min-out" : "expected-out",
            capturedAtMs: Date.now(),
          },
        },
      });
    }

    function getEffectivePreviewInputs() {
      return mergePreviewInputs(
        normalizePreviewInputs(getStoredPreviewInputs()),
        extractPreviewInputsFromStartupWarm(getStartupWarmPayload()),
      );
    }

    return {
      captureBonkUsd1ApproxFromReport,
      computeLocalQuote,
      extractPreviewInputsFromStartupWarm,
      mergePreviewInputs,
      normalizePreviewInputs: normalizePreviewInputsValue,
    };
  }

  function normalizeShape(shape) {
    const value = shape && typeof shape === "object" ? shape : {};
    return {
      launchpad: String(value.launchpad || "").trim().toLowerCase(),
      quoteAsset: String(value.quoteAsset || "").trim().toLowerCase() === "usd1" ? "usd1" : "sol",
      launchMode: String(value.launchMode || "").trim().toLowerCase(),
      mode: String(value.mode || "").trim().toLowerCase(),
      amount: String(value.amount || "").trim(),
    };
  }

  function buildPreviewInputsFromStartupWarm(payload) {
    const bundle = payload && typeof payload === "object" ? payload : {};
    const pumpPreviewBasis = bundle.pumpGlobal && typeof bundle.pumpGlobal === "object"
      ? bundle.pumpGlobal.previewBasis
      : null;
    const bonkPreviewBasis = bundle.bonkState && typeof bundle.bonkState === "object"
      ? bundle.bonkState.previewBasis
      : null;
    return {
      pump: pumpPreviewBasis || null,
      bonk: {
        defaultsByKey: indexBonkLaunchDefaults(bonkPreviewBasis && bonkPreviewBasis.launchDefaults),
      },
    };
  }

  function normalizePreviewInputs(value) {
    const source = value && typeof value === "object" ? value : {};
    return {
      schemaVersion: 1,
      pump: normalizePumpBasis(source.pump),
      bonk: {
        defaultsByKey: normalizeBonkDefaultsByKey(
          source.bonk && source.bonk.defaultsByKey ? source.bonk.defaultsByKey : null,
        ),
        usd1Approx: normalizeBonkUsd1Approx(
          source.bonk && source.bonk.usd1Approx ? source.bonk.usd1Approx : null,
        ),
      },
    };
  }

  function normalizePumpBasis(value) {
    const source = value && typeof value === "object" ? value : null;
    if (!source) return null;
    const basis = {
      initialVirtualTokenReserves: bigintStringOrEmpty(source.initialVirtualTokenReserves),
      initialVirtualSolReserves: bigintStringOrEmpty(source.initialVirtualSolReserves),
      initialRealTokenReserves: bigintStringOrEmpty(source.initialRealTokenReserves),
      feeBasisPoints: bigintStringOrEmpty(source.feeBasisPoints),
      creatorFeeBasisPoints: bigintStringOrEmpty(source.creatorFeeBasisPoints),
    };
    return basis.initialVirtualTokenReserves
      && basis.initialVirtualSolReserves
      && basis.initialRealTokenReserves
      && basis.feeBasisPoints
      && basis.creatorFeeBasisPoints
      ? basis
      : null;
  }

  function normalizeBonkDefaultsByKey(value) {
    const source = value && typeof value === "object" ? value : {};
    const result = {};
    Object.entries(source).forEach(([key, entry]) => {
      const normalized = normalizeBonkLaunchDefault(entry);
      if (normalized) result[key] = normalized;
    });
    return result;
  }

  function normalizeBonkLaunchDefault(value) {
    const source = value && typeof value === "object" ? value : null;
    if (!source) return null;
    const pool = source.pool && typeof source.pool === "object" ? source.pool : {};
    const normalized = {
      mode: String(source.mode || "").trim().toLowerCase(),
      quoteAsset: String(source.quoteAsset || "").trim().toLowerCase(),
      quoteAssetLabel: String(source.quoteAssetLabel || source.quoteAsset || "").trim() || "SOL",
      quoteDecimals: numberOrFallback(source.quoteDecimals, 9),
      supply: bigintStringOrEmpty(source.supply),
      totalFundRaisingB: bigintStringOrEmpty(source.totalFundRaisingB),
      tradeFeeRate: bigintStringOrEmpty(source.tradeFeeRate),
      platformFeeRate: bigintStringOrEmpty(source.platformFeeRate),
      creatorFeeRate: bigintStringOrEmpty(source.creatorFeeRate),
      curveType: numberOrFallback(source.curveType, 0),
      pool: {
        totalSellA: bigintStringOrEmpty(pool.totalSellA),
        virtualA: bigintStringOrEmpty(pool.virtualA),
        virtualB: bigintStringOrEmpty(pool.virtualB),
        realA: bigintStringOrEmpty(pool.realA),
        realB: bigintStringOrEmpty(pool.realB),
      },
    };
    return normalized.mode
      && normalized.quoteAsset
      && normalized.supply
      && normalized.tradeFeeRate
      && normalized.platformFeeRate
      && normalized.creatorFeeRate
      && normalized.pool.totalSellA
      && normalized.pool.virtualA
      && normalized.pool.virtualB
      && normalized.pool.realA
      && normalized.pool.realB
      ? normalized
      : null;
  }

  function normalizeBonkUsd1Approx(value) {
    const source = value && typeof value === "object" ? value : null;
    if (!source) return null;
    const inputLamports = bigintStringOrEmpty(source.inputLamports);
    const quoteOutAmount = bigintStringOrEmpty(source.quoteOutAmount);
    if (!inputLamports || !quoteOutAmount) return null;
    return {
      inputLamports,
      quoteOutAmount,
      source: String(source.source || "min-out").trim() || "min-out",
      capturedAtMs: numberOrFallback(source.capturedAtMs, 0),
    };
  }

  function indexBonkLaunchDefaults(entries) {
    const list = Array.isArray(entries) ? entries : [];
    const result = {};
    list.forEach((entry) => {
      const normalized = normalizeBonkLaunchDefault(entry);
      if (!normalized) return;
      result[`${normalized.mode}:${normalized.quoteAsset}`] = normalized;
    });
    return result;
  }

  function computePumpQuote(shape, basis, formatDecimal) {
    if (!basis) {
      return {
        quote: null,
        placeholder: "Preview unavailable until Pump warm state is ready.",
      };
    }
    const initialVirtualTokenReserves = BigInt(basis.initialVirtualTokenReserves);
    const initialVirtualSolReserves = BigInt(basis.initialVirtualSolReserves);
    const initialRealTokenReserves = BigInt(basis.initialRealTokenReserves);
    const feeBasisPoints = BigInt(basis.feeBasisPoints);
    const creatorFeeBasisPoints = BigInt(basis.creatorFeeBasisPoints);
    if (shape.mode === "sol") {
      const spendableSol = parseDecimalBigInt(shape.amount, 9, "buy amount");
      if (spendableSol <= 0n) return { quote: null, placeholder: "Enter a valid dev buy amount." };
      const totalFeeBasisPoints = feeBasisPoints + creatorFeeBasisPoints;
      const inputAmount = ((spendableSol - 1n) * 10_000n) / (10_000n + totalFeeBasisPoints);
      if (inputAmount <= 0n) return { quote: null, placeholder: "Enter a valid dev buy amount." };
      const tokensOut = minBigInt(
        (inputAmount * initialVirtualTokenReserves) / (initialVirtualSolReserves + inputAmount),
        initialRealTokenReserves,
      );
      return {
        quote: {
          mode: "sol",
          input: shape.amount,
          estimatedTokens: formatDecimal(tokensOut, PUMP_TOKEN_DECIMALS, 6),
          estimatedSol: formatDecimal(spendableSol, 9, 6),
          estimatedQuoteAmount: formatDecimal(spendableSol, 9, 6),
          quoteAsset: "sol",
          quoteAssetLabel: "SOL",
          estimatedSupplyPercent: formatSupplyPercent(tokensOut, PUMP_TOTAL_SUPPLY_RAW),
        },
      };
    }
    if (shape.mode === "tokens") {
      const tokenAmount = parseDecimalBigInt(shape.amount, PUMP_TOKEN_DECIMALS, "buy amount");
      if (tokenAmount <= 0n || tokenAmount >= initialVirtualTokenReserves) {
        return { quote: null, placeholder: "Enter a valid dev buy amount." };
      }
      const solCost = ((tokenAmount * initialVirtualSolReserves) / (initialVirtualTokenReserves - tokenAmount)) + 1n;
      const protocolFee = ceilDiv(solCost * feeBasisPoints, 10_000n);
      const creatorFee = ceilDiv(solCost * creatorFeeBasisPoints, 10_000n);
      const totalSol = solCost + protocolFee + creatorFee;
      return {
        quote: {
          mode: "tokens",
          input: shape.amount,
          estimatedTokens: formatDecimal(tokenAmount, PUMP_TOKEN_DECIMALS, 6),
          estimatedSol: formatDecimal(totalSol, 9, 6),
          estimatedQuoteAmount: formatDecimal(totalSol, 9, 6),
          quoteAsset: "sol",
          quoteAssetLabel: "SOL",
          estimatedSupplyPercent: formatSupplyPercent(tokenAmount, PUMP_TOTAL_SUPPLY_RAW),
        },
      };
    }
    return {
      quote: null,
      placeholder: "Unsupported Pump preview mode.",
    };
  }

  function computeBonkQuote(shape, bonkState, formatDecimal) {
    const defaults = bonkState && bonkState.defaultsByKey
      ? bonkState.defaultsByKey[`${shape.launchMode}:${shape.quoteAsset}`]
      : null;
    if (!defaults) {
      return {
        quote: null,
        placeholder: "Preview unavailable until Bonk launch defaults are ready.",
      };
    }
    const approximateUsd1 = defaults.quoteAsset === "usd1"
      ? (bonkState && bonkState.usd1Approx ? bonkState.usd1Approx : null)
      : null;
    if (shape.mode === "sol") {
      const inputSol = parseDecimalBigInt(shape.amount, 9, "buy amount");
      if (inputSol <= 0n) return { quote: null, placeholder: "Enter a valid dev buy amount." };
      let quoteAmountB;
      let approximate = false;
      if (defaults.quoteAsset === "usd1") {
        if (!approximateUsd1) {
          return {
            quote: null,
            placeholder: "Preview unavailable until a Bonk USD1 route sample is available.",
          };
        }
        quoteAmountB = (inputSol * BigInt(approximateUsd1.quoteOutAmount)) / BigInt(approximateUsd1.inputLamports);
        approximate = true;
      } else {
        quoteAmountB = parseDecimalBigInt(shape.amount, defaults.quoteDecimals, `buy amount ${defaults.quoteAssetLabel}`);
      }
      const tokenAmount = bonkQuoteBuyExactInAmountA(defaults, quoteAmountB);
      return {
        quote: {
          mode: "sol",
          input: shape.amount,
          estimatedTokens: formatDecimal(tokenAmount, BONK_TOKEN_DECIMALS, 6),
          estimatedSol: formatDecimal(inputSol, 9, 6),
          estimatedQuoteAmount: formatDecimal(inputSol, 9, 6),
          quoteAsset: "sol",
          quoteAssetLabel: "SOL",
          estimatedSupplyPercent: formatSupplyPercent(tokenAmount, BigInt(defaults.supply), 6),
          previewOnly: approximate,
        },
      };
    }
    if (shape.mode === "tokens") {
      const tokenAmount = parseDecimalBigInt(shape.amount, BONK_TOKEN_DECIMALS, "buy amount");
      if (tokenAmount <= 0n) return { quote: null, placeholder: "Enter a valid dev buy amount." };
      const requiredQuoteAmount = bonkQuoteBuyExactOutAmountB(defaults, tokenAmount);
      let requiredSol = requiredQuoteAmount;
      let approximate = false;
      if (defaults.quoteAsset === "usd1") {
        if (!approximateUsd1) {
          return {
            quote: null,
            placeholder: "Preview unavailable until a Bonk USD1 route sample is available.",
          };
        }
        requiredSol = ceilDiv(
          requiredQuoteAmount * BigInt(approximateUsd1.inputLamports),
          BigInt(approximateUsd1.quoteOutAmount),
        );
        approximate = true;
      }
      return {
        quote: {
          mode: "tokens",
          input: shape.amount,
          estimatedTokens: formatDecimal(tokenAmount, BONK_TOKEN_DECIMALS, 6),
          estimatedSol: formatDecimal(requiredSol, 9, 6),
          estimatedQuoteAmount: formatDecimal(requiredSol, 9, 6),
          quoteAsset: "sol",
          quoteAssetLabel: "SOL",
          estimatedSupplyPercent: formatSupplyPercent(tokenAmount, BigInt(defaults.supply), 6),
          previewOnly: approximate,
        },
      };
    }
    return {
      quote: null,
      placeholder: "Unsupported Bonk preview mode.",
    };
  }

  function computeBagsQuote(shape, formatDecimal) {
    if (shape.mode === "sol") {
      const buyAmountLamports = parseDecimalBigInt(shape.amount, 9, "buy amount");
      if (buyAmountLamports <= 0n) return { quote: null, placeholder: "Enter a valid dev buy amount." };
      const feeNumerator = bagsCliffFeeNumeratorForMode(shape.launchMode);
      const afterFee = bagsGetFeeAmountExcluded(buyAmountLamports, feeNumerator);
      const outputTokens = bagsGetQuoteToBaseOutput(afterFee);
      return {
        quote: {
          mode: "sol",
          input: shape.amount,
          estimatedTokens: formatDecimal(outputTokens, BAGS_TOKEN_DECIMALS, 6),
          estimatedSol: formatDecimal(buyAmountLamports, 9, 6),
          estimatedQuoteAmount: formatDecimal(buyAmountLamports, 9, 6),
          quoteAsset: "sol",
          quoteAssetLabel: "SOL",
          estimatedSupplyPercent: formatSupplyPercent(outputTokens, BAGS_TOTAL_SUPPLY_RAW),
        },
      };
    }
    if (shape.mode === "tokens") {
      const desiredTokens = parseDecimalBigInt(shape.amount, BAGS_TOKEN_DECIMALS, "buy amount");
      if (desiredTokens <= 0n) return { quote: null, placeholder: "Enter a valid dev buy amount." };
      const feeNumerator = bagsCliffFeeNumeratorForMode(shape.launchMode);
      const excludedInput = bagsGetQuoteToBaseInputForOutput(desiredTokens);
      const requiredInput = bagsGetFeeAmountIncluded(excludedInput, feeNumerator);
      return {
        quote: {
          mode: "tokens",
          input: shape.amount,
          estimatedTokens: formatDecimal(desiredTokens, BAGS_TOKEN_DECIMALS, 6),
          estimatedSol: formatDecimal(requiredInput, 9, 6),
          estimatedQuoteAmount: formatDecimal(requiredInput, 9, 6),
          quoteAsset: "sol",
          quoteAssetLabel: "SOL",
          estimatedSupplyPercent: formatSupplyPercent(desiredTokens, BAGS_TOTAL_SUPPLY_RAW),
        },
      };
    }
    return {
      quote: null,
      placeholder: "Unsupported Bags preview mode.",
    };
  }

  function bonkQuoteBuyExactInAmountA(defaults, amountB) {
    const feeRate = bonkTotalFeeRate(defaults);
    const totalFee = bonkCalculateFee(amountB, feeRate);
    const amountLessFeeB = bonkBigSub(amountB, totalFee, "buy input after fee");
    const quotedAmountA = bonkCurveBuyExactIn(defaults, amountLessFeeB);
    const remainingAmountA = bonkBigSub(BigInt(defaults.pool.totalSellA), BigInt(defaults.pool.realA), "remaining sell amount");
    return quotedAmountA > remainingAmountA ? remainingAmountA : quotedAmountA;
  }

  function bonkQuoteBuyExactOutAmountB(defaults, requestedAmountA) {
    const remainingAmountA = bonkBigSub(BigInt(defaults.pool.totalSellA), BigInt(defaults.pool.realA), "remaining sell amount");
    const realAmountA = requestedAmountA > remainingAmountA ? remainingAmountA : requestedAmountA;
    const amountInLessFeeB = bonkCurveBuyExactOut(defaults, realAmountA);
    return bonkCalculatePreFee(amountInLessFeeB, bonkTotalFeeRate(defaults));
  }

  function bonkCurveBuyExactIn(defaults, amount) {
    const pool = defaults.pool;
    const curveType = Number(defaults.curveType || 0);
    if (curveType === 0) {
      const inputReserve = BigInt(pool.virtualB) + BigInt(pool.realB);
      const outputReserve = bonkBigSub(BigInt(pool.virtualA), BigInt(pool.realA), "launch output reserve");
      return (amount * outputReserve) / (inputReserve + amount);
    }
    if (curveType === 1) {
      const virtualB = BigInt(pool.virtualB);
      if (virtualB === 0n) throw new Error("Bonk fixed-price virtual quote reserve was zero.");
      return (BigInt(pool.virtualA) * amount) / virtualB;
    }
    if (curveType === 2) {
      const virtualA = BigInt(pool.virtualA);
      if (virtualA === 0n) throw new Error("Bonk linear-price virtual coefficient was zero.");
      const newQuote = BigInt(pool.realB) + amount;
      const termInsideSqrt = (2n * newQuote * BONK_Q64) / virtualA;
      const sqrtTerm = bigIntSqrtRound(termInsideSqrt);
      return bonkBigSub(sqrtTerm, BigInt(pool.realA), "linear-price amount out");
    }
    throw new Error("Unsupported Bonk curve type.");
  }

  function bonkCurveBuyExactOut(defaults, amount) {
    const pool = defaults.pool;
    const curveType = Number(defaults.curveType || 0);
    if (curveType === 0) {
      const inputReserve = BigInt(pool.virtualB) + BigInt(pool.realB);
      const outputReserve = bonkBigSub(BigInt(pool.virtualA), BigInt(pool.realA), "launch output reserve");
      const denominator = bonkBigSub(outputReserve, amount, "launch remaining output reserve");
      if (denominator === 0n) throw new Error("Bonk constant-product buyExactOut denominator was zero.");
      return ceilDiv(inputReserve * amount, denominator);
    }
    if (curveType === 1) {
      const virtualA = BigInt(pool.virtualA);
      if (virtualA === 0n) throw new Error("Bonk fixed-price virtual token reserve was zero.");
      return ceilDiv(BigInt(pool.virtualB) * amount, virtualA);
    }
    if (curveType === 2) {
      const newBase = BigInt(pool.realA) + amount;
      const newBaseSquared = newBase * newBase;
      const denominator = 2n * BONK_Q64;
      const newQuote = ceilDiv(BigInt(pool.virtualA) * newBaseSquared, denominator);
      return bonkBigSub(newQuote, BigInt(pool.realB), "linear-price amount in");
    }
    throw new Error("Unsupported Bonk curve type.");
  }

  function bonkTotalFeeRate(defaults) {
    const total = BigInt(defaults.tradeFeeRate) + BigInt(defaults.platformFeeRate) + BigInt(defaults.creatorFeeRate);
    if (total > BONK_FEE_RATE_DENOMINATOR) {
      throw new Error("Bonk total fee rate exceeded denominator.");
    }
    return total;
  }

  function bonkCalculateFee(amount, feeRate) {
    return ceilDiv(amount * feeRate, BONK_FEE_RATE_DENOMINATOR);
  }

  function bonkCalculatePreFee(postFeeAmount, feeRate) {
    if (feeRate === 0n) return postFeeAmount;
    const denominator = bonkBigSub(BONK_FEE_RATE_DENOMINATOR, feeRate, "fee denominator");
    if (denominator === 0n) throw new Error("Bonk fee denominator was zero.");
    return ceilDiv(postFeeAmount * BONK_FEE_RATE_DENOMINATOR, denominator);
  }

  function bonkBigSub(left, right, label) {
    if (left < right) {
      throw new Error(`Bonk ${label} underflow.`);
    }
    return left - right;
  }

  function bagsCliffFeeNumeratorForMode(mode) {
    const normalized = String(mode || "").trim().toLowerCase();
    if (normalized === "bags-025-1") return 25n * BAGS_FEE_DENOMINATOR / 10_000n;
    if (normalized === "bags-1-025") return 100n * BAGS_FEE_DENOMINATOR / 10_000n;
    return 200n * BAGS_FEE_DENOMINATOR / 10_000n;
  }

  function bagsGetQuoteToBaseOutput(amountIn) {
    let totalOutput = 0n;
    let sqrtPrice = BAGS_INITIAL_SQRT_PRICE;
    let amountLeft = amountIn;
    for (const point of BAGS_CURVE_POINTS) {
      if (point.sqrtPrice <= sqrtPrice || point.sqrtPrice === 0n || point.liquidity === 0n) continue;
      const maxAmountIn = bagsGetDeltaAmountQuoteUnsigned(sqrtPrice, point.sqrtPrice, point.liquidity, true);
      if (amountLeft < maxAmountIn) {
        const nextSqrtPrice = bagsGetNextSqrtPriceFromInput(sqrtPrice, point.liquidity, amountLeft);
        totalOutput += bagsGetDeltaAmountBaseUnsigned(sqrtPrice, nextSqrtPrice, point.liquidity, false);
        amountLeft = 0n;
        break;
      }
      totalOutput += bagsGetDeltaAmountBaseUnsigned(sqrtPrice, point.sqrtPrice, point.liquidity, false);
      sqrtPrice = point.sqrtPrice;
      amountLeft = bagsBigSub(amountLeft, maxAmountIn, "remaining quote input");
    }
    if (amountLeft !== 0n) {
      throw new Error("Not enough liquidity to process the entire Bags amount.");
    }
    return totalOutput;
  }

  function bagsGetQuoteToBaseInputForOutput(outAmount) {
    let totalInput = 0n;
    let sqrtPrice = BAGS_INITIAL_SQRT_PRICE;
    let amountLeft = outAmount;
    for (const point of BAGS_CURVE_POINTS) {
      if (point.sqrtPrice <= sqrtPrice || point.sqrtPrice === 0n || point.liquidity === 0n) continue;
      const maxAmountOut = bagsGetDeltaAmountBaseUnsigned(sqrtPrice, point.sqrtPrice, point.liquidity, false);
      if (amountLeft < maxAmountOut) {
        const nextSqrtPrice = bagsGetNextSqrtPriceFromBaseOutput(sqrtPrice, point.liquidity, amountLeft);
        totalInput += bagsGetDeltaAmountQuoteUnsigned(sqrtPrice, nextSqrtPrice, point.liquidity, true);
        amountLeft = 0n;
        break;
      }
      totalInput += bagsGetDeltaAmountQuoteUnsigned(sqrtPrice, point.sqrtPrice, point.liquidity, true);
      sqrtPrice = point.sqrtPrice;
      amountLeft = bagsBigSub(amountLeft, maxAmountOut, "remaining base output");
    }
    if (amountLeft !== 0n) {
      throw new Error("Not enough liquidity for the requested Bags amount.");
    }
    return totalInput;
  }

  function bagsGetDeltaAmountBaseUnsigned(lowerSqrtPrice, upperSqrtPrice, liquidity, roundUp) {
    if (liquidity === 0n) return 0n;
    if (lowerSqrtPrice === 0n || upperSqrtPrice === 0n) {
      throw new Error("Bags quote sqrt price cannot be zero.");
    }
    const numerator = bagsBigSub(upperSqrtPrice, lowerSqrtPrice, "base numerator");
    const denominator = lowerSqrtPrice * upperSqrtPrice;
    return divRounding(liquidity * numerator, denominator, roundUp);
  }

  function bagsGetDeltaAmountQuoteUnsigned(lowerSqrtPrice, upperSqrtPrice, liquidity, roundUp) {
    if (liquidity === 0n) return 0n;
    const delta = bagsBigSub(upperSqrtPrice, lowerSqrtPrice, "quote numerator");
    const denominator = 1n << (BAGS_RESOLUTION_BITS * 2n);
    return divRounding(liquidity * delta, denominator, roundUp);
  }

  function bagsGetNextSqrtPriceFromInput(sqrtPrice, liquidity, amountIn) {
    if (sqrtPrice === 0n || liquidity === 0n) {
      throw new Error("Bags quote price or liquidity cannot be zero.");
    }
    if (amountIn === 0n) return sqrtPrice;
    return sqrtPrice + ((amountIn << (BAGS_RESOLUTION_BITS * 2n)) / liquidity);
  }

  function bagsGetNextSqrtPriceFromBaseOutput(sqrtPrice, liquidity, amountOut) {
    if (sqrtPrice === 0n) {
      throw new Error("Bags quote sqrt price cannot be zero.");
    }
    if (amountOut === 0n) return sqrtPrice;
    const denominator = bagsBigSub(liquidity, amountOut * sqrtPrice, "next sqrt denominator");
    return divRounding(liquidity * sqrtPrice, denominator, false);
  }

  function bagsGetFeeAmountIncluded(amount, feeNumerator) {
    if (feeNumerator === 0n) return amount;
    return divRounding(amount * BAGS_FEE_DENOMINATOR, BAGS_FEE_DENOMINATOR - feeNumerator, true);
  }

  function bagsGetFeeAmountExcluded(amount, feeNumerator) {
    if (feeNumerator === 0n) return amount;
    const tradingFee = divRounding(amount * feeNumerator, BAGS_FEE_DENOMINATOR, true);
    return amount - tradingFee;
  }

  function bagsBigSub(left, right, label) {
    if (left < right) {
      throw new Error(`Bags math underflow while computing ${label}.`);
    }
    return left - right;
  }

  function divRounding(numerator, denominator, roundUp) {
    if (!roundUp || numerator === 0n) return numerator / denominator;
    return (numerator + denominator - 1n) / denominator;
  }

  function ceilDiv(numerator, denominator) {
    if (numerator === 0n) return 0n;
    return (numerator + denominator - 1n) / denominator;
  }

  function minBigInt(left, right) {
    return left <= right ? left : right;
  }

  function formatSupplyPercent(amountRaw, totalSupplyRaw, decimals = 4) {
    if (amountRaw <= 0n || totalSupplyRaw <= 0n) return "0";
    const scaled = (amountRaw * 100_000_000n) / totalSupplyRaw;
    return defaultFormatBigIntDecimal(scaled, 6, decimals);
  }

  function parseDecimalBigInt(rawValue, decimals, label) {
    const raw = String(rawValue || "").trim();
    if (!raw) throw new Error(`${label} is required.`);
    if (!/^\d+(\.\d+)?$/.test(raw)) {
      throw new Error(`Invalid ${label}.`);
    }
    const [whole, fraction = ""] = raw.split(".");
    if (fraction.length > decimals) {
      throw new Error(`Too many decimal places (max ${decimals}).`);
    }
    const paddedFraction = `${fraction}${"0".repeat(decimals)}`.slice(0, decimals);
    return (BigInt(whole || "0") * (10n ** BigInt(decimals))) + BigInt(paddedFraction || "0");
  }

  function decimalStringToRaw(value, decimals) {
    if (value == null || value === "") return "";
    try {
      return parseDecimalBigInt(value, decimals, "value").toString();
    } catch (_error) {
      return "";
    }
  }

  function bigintStringOrEmpty(value) {
    if (typeof value === "bigint") return value.toString();
    if (typeof value === "number" && Number.isFinite(value) && value >= 0) return String(Math.trunc(value));
    if (typeof value !== "string") return "";
    const trimmed = value.trim();
    return /^\d+$/.test(trimmed) ? trimmed : "";
  }

  function numberOrFallback(value, fallback) {
    const numeric = Number(value);
    return Number.isFinite(numeric) ? numeric : fallback;
  }

  function bigIntSqrtFloor(value) {
    if (value <= 1n) return value;
    let current = 1n << BigInt(Math.ceil(value.toString(2).length / 2));
    while (true) {
      const next = (current + (value / current)) >> 1n;
      if (next >= current) return current;
      current = next;
    }
  }

  function bigIntSqrtRound(value) {
    const floor = bigIntSqrtFloor(value);
    const remainder = value - (floor * floor);
    return remainder > floor ? floor + 1n : floor;
  }

  function defaultFormatBigIntDecimal(value, decimals, maxFractionDigits) {
    const negative = value < 0n;
    const absolute = negative ? -value : value;
    const base = 10n ** BigInt(decimals);
    const whole = absolute / base;
    const fraction = absolute % base;
    if (fraction === 0n) return `${negative ? "-" : ""}${whole.toString()}`;
    let fractionText = fraction.toString().padStart(decimals, "0").slice(0, maxFractionDigits);
    fractionText = fractionText.replace(/0+$/, "");
    return `${negative ? "-" : ""}${whole.toString()}${fractionText ? `.${fractionText}` : ""}`;
  }

  global.LaunchDeckQuotePreviewDomain = {
    create,
  };
})(window);
