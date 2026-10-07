package org.tensorflow.op;

import java.nio.charset.Charset;
import java.util.List;
import org.tensorflow.DataType;
import org.tensorflow.EagerSession;
import org.tensorflow.ExecutionEnvironment;
import org.tensorflow.Operand;
import org.tensorflow.Tensor;
import org.tensorflow.op.core.Abort;
import org.tensorflow.op.core.All;
import org.tensorflow.op.core.Any;
import org.tensorflow.op.core.AssertThat;
import org.tensorflow.op.core.Assign;
import org.tensorflow.op.core.AssignAdd;
import org.tensorflow.op.core.AssignAddVariableOp;
import org.tensorflow.op.core.AssignSub;
import org.tensorflow.op.core.AssignSubVariableOp;
import org.tensorflow.op.core.AssignVariableOp;
import org.tensorflow.op.core.Barrier;
import org.tensorflow.op.core.BarrierClose;
import org.tensorflow.op.core.BarrierIncompleteSize;
import org.tensorflow.op.core.BarrierInsertMany;
import org.tensorflow.op.core.BarrierReadySize;
import org.tensorflow.op.core.BarrierTakeMany;
import org.tensorflow.op.core.Batch;
import org.tensorflow.op.core.BatchToSpace;
import org.tensorflow.op.core.BatchToSpaceNd;
import org.tensorflow.op.core.Bitcast;
import org.tensorflow.op.core.BroadcastDynamicShape;
import org.tensorflow.op.core.BroadcastTo;
import org.tensorflow.op.core.Bucketize;
import org.tensorflow.op.core.ClipByValue;
import org.tensorflow.op.core.Concat;
import org.tensorflow.op.core.Constant;
import org.tensorflow.op.core.ConsumeMutexLock;
import org.tensorflow.op.core.ControlTrigger;
import org.tensorflow.op.core.CountUpTo;
import org.tensorflow.op.core.DeepCopy;
import org.tensorflow.op.core.DeleteSessionTensor;
import org.tensorflow.op.core.DestroyResourceOp;
import org.tensorflow.op.core.DestroyTemporaryVariable;
import org.tensorflow.op.core.DynamicPartition;
import org.tensorflow.op.core.DynamicStitch;
import org.tensorflow.op.core.EditDistance;
import org.tensorflow.op.core.Empty;
import org.tensorflow.op.core.EmptyTensorList;
import org.tensorflow.op.core.EnsureShape;
import org.tensorflow.op.core.ExpandDims;
import org.tensorflow.op.core.ExtractVolumePatches;
import org.tensorflow.op.core.Fill;
import org.tensorflow.op.core.Fingerprint;
import org.tensorflow.op.core.Gather;
import org.tensorflow.op.core.GatherNd;
import org.tensorflow.op.core.GetSessionHandle;
import org.tensorflow.op.core.GetSessionTensor;
import org.tensorflow.op.core.Gradients;
import org.tensorflow.op.core.GuaranteeConst;
import org.tensorflow.op.core.HashTable;
import org.tensorflow.op.core.Helpers;
import org.tensorflow.op.core.HistogramFixedWidth;
import org.tensorflow.op.core.Identity;
import org.tensorflow.op.core.IdentityN;
import org.tensorflow.op.core.ImmutableConst;
import org.tensorflow.op.core.Init;
import org.tensorflow.op.core.InitializeTable;
import org.tensorflow.op.core.InitializeTableFromTextFile;
import org.tensorflow.op.core.InplaceAdd;
import org.tensorflow.op.core.InplaceSub;
import org.tensorflow.op.core.InplaceUpdate;
import org.tensorflow.op.core.IsVariableInitialized;
import org.tensorflow.op.core.LinSpace;
import org.tensorflow.op.core.LookupTableExport;
import org.tensorflow.op.core.LookupTableFind;
import org.tensorflow.op.core.LookupTableImport;
import org.tensorflow.op.core.LookupTableInsert;
import org.tensorflow.op.core.LookupTableSize;
import org.tensorflow.op.core.LoopCond;
import org.tensorflow.op.core.MapClear;
import org.tensorflow.op.core.MapIncompleteSize;
import org.tensorflow.op.core.MapPeek;
import org.tensorflow.op.core.MapSize;
import org.tensorflow.op.core.MapStage;
import org.tensorflow.op.core.MapUnstage;
import org.tensorflow.op.core.MapUnstageNoKey;
import org.tensorflow.op.core.Max;
import org.tensorflow.op.core.Merge;
import org.tensorflow.op.core.Min;
import org.tensorflow.op.core.MirrorPad;
import org.tensorflow.op.core.MlirPassthroughOp;
import org.tensorflow.op.core.MutableDenseHashTable;
import org.tensorflow.op.core.MutableHashTable;
import org.tensorflow.op.core.MutableHashTableOfTensors;
import org.tensorflow.op.core.Mutex;
import org.tensorflow.op.core.MutexLock;
import org.tensorflow.op.core.NextIteration;
import org.tensorflow.op.core.NoOp;
import org.tensorflow.op.core.OneHot;
import org.tensorflow.op.core.OnesLike;
import org.tensorflow.op.core.OrderedMapClear;
import org.tensorflow.op.core.OrderedMapIncompleteSize;
import org.tensorflow.op.core.OrderedMapPeek;
import org.tensorflow.op.core.OrderedMapSize;
import org.tensorflow.op.core.OrderedMapStage;
import org.tensorflow.op.core.OrderedMapUnstage;
import org.tensorflow.op.core.OrderedMapUnstageNoKey;
import org.tensorflow.op.core.Pad;
import org.tensorflow.op.core.ParallelConcat;
import org.tensorflow.op.core.ParallelDynamicStitch;
import org.tensorflow.op.core.Placeholder;
import org.tensorflow.op.core.PlaceholderWithDefault;
import org.tensorflow.op.core.Print;
import org.tensorflow.op.core.Prod;
import org.tensorflow.op.core.QuantizedReshape;
import org.tensorflow.op.core.Range;
import org.tensorflow.op.core.Rank;
import org.tensorflow.op.core.ReadVariableOp;
import org.tensorflow.op.core.ReduceAll;
import org.tensorflow.op.core.ReduceAny;
import org.tensorflow.op.core.ReduceMax;
import org.tensorflow.op.core.ReduceMin;
import org.tensorflow.op.core.ReduceProd;
import org.tensorflow.op.core.ReduceSum;
import org.tensorflow.op.core.RefNextIteration;
import org.tensorflow.op.core.RefSelect;
import org.tensorflow.op.core.RefSwitch;
import org.tensorflow.op.core.RemoteFusedGraphExecute;
import org.tensorflow.op.core.Reshape;
import org.tensorflow.op.core.ResourceCountUpTo;
import org.tensorflow.op.core.ResourceGather;
import org.tensorflow.op.core.ResourceGatherNd;
import org.tensorflow.op.core.ResourceScatterAdd;
import org.tensorflow.op.core.ResourceScatterDiv;
import org.tensorflow.op.core.ResourceScatterMax;
import org.tensorflow.op.core.ResourceScatterMin;
import org.tensorflow.op.core.ResourceScatterMul;
import org.tensorflow.op.core.ResourceScatterNdAdd;
import org.tensorflow.op.core.ResourceScatterNdSub;
import org.tensorflow.op.core.ResourceScatterNdUpdate;
import org.tensorflow.op.core.ResourceScatterSub;
import org.tensorflow.op.core.ResourceScatterUpdate;
import org.tensorflow.op.core.ResourceStridedSliceAssign;
import org.tensorflow.op.core.Reverse;
import org.tensorflow.op.core.ReverseSequence;
import org.tensorflow.op.core.Roll;
import org.tensorflow.op.core.Rpc;
import org.tensorflow.op.core.ScatterAdd;
import org.tensorflow.op.core.ScatterDiv;
import org.tensorflow.op.core.ScatterMax;
import org.tensorflow.op.core.ScatterMin;
import org.tensorflow.op.core.ScatterMul;
import org.tensorflow.op.core.ScatterNd;
import org.tensorflow.op.core.ScatterNdAdd;
import org.tensorflow.op.core.ScatterNdNonAliasingAdd;
import org.tensorflow.op.core.ScatterNdSub;
import org.tensorflow.op.core.ScatterNdUpdate;
import org.tensorflow.op.core.ScatterSub;
import org.tensorflow.op.core.ScatterUpdate;
import org.tensorflow.op.core.Select;
import org.tensorflow.op.core.SetDiff1d;
import org.tensorflow.op.core.SetSize;
import org.tensorflow.op.core.ShapeN;
import org.tensorflow.op.core.Size;
import org.tensorflow.op.core.Skipgram;
import org.tensorflow.op.core.Slice;
import org.tensorflow.op.core.Snapshot;
import org.tensorflow.op.core.SpaceToBatchNd;
import org.tensorflow.op.core.Split;
import org.tensorflow.op.core.SplitV;
import org.tensorflow.op.core.Squeeze;
import org.tensorflow.op.core.Stack;
import org.tensorflow.op.core.Stage;
import org.tensorflow.op.core.StageClear;
import org.tensorflow.op.core.StagePeek;
import org.tensorflow.op.core.StageSize;
import org.tensorflow.op.core.StopGradient;
import org.tensorflow.op.core.StridedSlice;
import org.tensorflow.op.core.StridedSliceAssign;
import org.tensorflow.op.core.StridedSliceGrad;
import org.tensorflow.op.core.Sum;
import org.tensorflow.op.core.SwitchCond;
import org.tensorflow.op.core.TemporaryVariable;
import org.tensorflow.op.core.TensorArray;
import org.tensorflow.op.core.TensorArrayClose;
import org.tensorflow.op.core.TensorArrayConcat;
import org.tensorflow.op.core.TensorArrayGather;
import org.tensorflow.op.core.TensorArrayGrad;
import org.tensorflow.op.core.TensorArrayGradWithShape;
import org.tensorflow.op.core.TensorArrayPack;
import org.tensorflow.op.core.TensorArrayRead;
import org.tensorflow.op.core.TensorArrayScatter;
import org.tensorflow.op.core.TensorArraySize;
import org.tensorflow.op.core.TensorArraySplit;
import org.tensorflow.op.core.TensorArrayUnpack;
import org.tensorflow.op.core.TensorArrayWrite;
import org.tensorflow.op.core.TensorListConcat;
import org.tensorflow.op.core.TensorListConcatLists;
import org.tensorflow.op.core.TensorListElementShape;
import org.tensorflow.op.core.TensorListFromTensor;
import org.tensorflow.op.core.TensorListGather;
import org.tensorflow.op.core.TensorListGetItem;
import org.tensorflow.op.core.TensorListLength;
import org.tensorflow.op.core.TensorListPopBack;
import org.tensorflow.op.core.TensorListPushBack;
import org.tensorflow.op.core.TensorListPushBackBatch;
import org.tensorflow.op.core.TensorListReserve;
import org.tensorflow.op.core.TensorListResize;
import org.tensorflow.op.core.TensorListScatter;
import org.tensorflow.op.core.TensorListScatterIntoExistingList;
import org.tensorflow.op.core.TensorListSetItem;
import org.tensorflow.op.core.TensorListSplit;
import org.tensorflow.op.core.TensorListStack;
import org.tensorflow.op.core.TensorScatterNdAdd;
import org.tensorflow.op.core.TensorScatterNdSub;
import org.tensorflow.op.core.TensorScatterNdUpdate;
import org.tensorflow.op.core.TensorStridedSliceUpdate;
import org.tensorflow.op.core.Tile;
import org.tensorflow.op.core.Timestamp;
import org.tensorflow.op.core.TryRpc;
import org.tensorflow.op.core.Unbatch;
import org.tensorflow.op.core.UnbatchGrad;
import org.tensorflow.op.core.Unique;
import org.tensorflow.op.core.UniqueWithCounts;
import org.tensorflow.op.core.UnravelIndex;
import org.tensorflow.op.core.Unstack;
import org.tensorflow.op.core.Unstage;
import org.tensorflow.op.core.VarHandleOp;
import org.tensorflow.op.core.VarIsInitializedOp;
import org.tensorflow.op.core.Variable;
import org.tensorflow.op.core.VariableShape;
import org.tensorflow.op.core.Where;
import org.tensorflow.op.core.Zeros;
import org.tensorflow.op.core.ZerosLike;
import org.tensorflow.tools.Shape;
import org.tensorflow.tools.buffer.BooleanDataBuffer;
import org.tensorflow.tools.buffer.ByteDataBuffer;
import org.tensorflow.tools.buffer.DataBuffer;
import org.tensorflow.tools.buffer.DoubleDataBuffer;
import org.tensorflow.tools.buffer.FloatDataBuffer;
import org.tensorflow.tools.buffer.IntDataBuffer;
import org.tensorflow.tools.buffer.LongDataBuffer;
import org.tensorflow.tools.ndarray.BooleanNdArray;
import org.tensorflow.tools.ndarray.ByteNdArray;
import org.tensorflow.tools.ndarray.DoubleNdArray;
import org.tensorflow.tools.ndarray.FloatNdArray;
import org.tensorflow.tools.ndarray.IntNdArray;
import org.tensorflow.tools.ndarray.LongNdArray;
import org.tensorflow.tools.ndarray.NdArray;
import org.tensorflow.types.TBool;
import org.tensorflow.types.TFloat32;
import org.tensorflow.types.TFloat64;
import org.tensorflow.types.TInt32;
import org.tensorflow.types.TInt64;
import org.tensorflow.types.TString;
import org.tensorflow.types.TUint8;
import org.tensorflow.types.family.TNumber;
import org.tensorflow.types.family.TType;

public final class Ops {
    public final NnOps nn;
    public final SummaryOps summary;
    public final ImageOps image;
    public final DataOps data;
    public final IoOps io;
    public final DtypesOps dtypes;
    public final LinalgOps linalg;
    public final RandomOps random;
    public final StringsOps strings;
    public final SparseOps sparse;
    public final BitwiseOps bitwise;
    public final MathOps math;
    public final AudioOps audio;
    public final SignalOps signal;
    public final TrainOps train;
    public final QuantizationOps quantization;
    private final Scope scope;
    private Ops(Scope scope) {
        this.scope = scope;
        nn = new NnOps(scope);
        summary = new SummaryOps(scope);
        image = new ImageOps(scope);
        data = new DataOps(scope);
        io = new IoOps(scope);
        dtypes = new DtypesOps(scope);
        linalg = new LinalgOps(scope);
        random = new RandomOps(scope);
        strings = new StringsOps(scope);
        sparse = new SparseOps(scope);
        bitwise = new BitwiseOps(scope);
        math = new MathOps(scope);
        audio = new AudioOps(scope);
        signal = new SignalOps(scope);
        train = new TrainOps(scope);
        quantization = new QuantizationOps(scope);
    }
    public Abort abort(Abort.Options... options) {
        return Abort.create(scope, options);
    }
    public <T extends TNumber> All all(Operand<TBool> input, Operand<T> axis, All.Options... options) {
        return All.create(scope, input, axis, options);
    }
    public <T extends TNumber> Any any(Operand<TBool> input, Operand<T> axis, Any.Options... options) {
        return Any.create(scope, input, axis, options);
    }
    public Constant<TInt32> array(int... data) {
        return Constant.arrayOf(scope, data);
    }
    public Constant<TString> array(String... data) {
        return Constant.arrayOf(scope, data);
    }
    public Constant<TBool> array(boolean... data) {
        return Constant.arrayOf(scope, data);
    }
    public Constant<TInt64> array(long... data) {
        return Constant.arrayOf(scope, data);
    }
    public Constant<TFloat32> array(float... data) {
        return Constant.arrayOf(scope, data);
    }
    public Constant<TFloat64> array(double... data) {
        return Constant.arrayOf(scope, data);
    }
    public Constant<TUint8> array(byte... data) {
        return Constant.arrayOf(scope, data);
    }
    public Constant<TString> array(Charset charset, String... data) {
        return Constant.arrayOf(scope, charset, data);
    }
    public AssertThat assertThat(Operand<TBool> condition, Iterable<Operand<?>> data, AssertThat.Options... options) {
        return AssertThat.create(scope, condition, data, options);
    }
    public <T extends TType> Assign<T> assign(Operand<T> ref, Operand<T> value, Assign.Options... options) {
        return Assign.create(scope, ref, value, options);
    }
    public <T extends TType> AssignAdd<T> assignAdd(Operand<T> ref, Operand<T> value, AssignAdd.Options... options) {
        return AssignAdd.create(scope, ref, value, options);
    }
    public <T extends TType> AssignAddVariableOp assignAddVariableOp(Operand<?> resource, Operand<T> value) {
        return AssignAddVariableOp.create(scope, resource, value);
    }
    public <T extends TType> AssignSub<T> assignSub(Operand<T> ref, Operand<T> value, AssignSub.Options... options) {
        return AssignSub.create(scope, ref, value, options);
    }
    public <T extends TType> AssignSubVariableOp assignSubVariableOp(Operand<?> resource, Operand<T> value) {
        return AssignSubVariableOp.create(scope, resource, value);
    }
    public <T extends TType> AssignVariableOp assignVariableOp(Operand<?> resource, Operand<T> value) {
        return AssignVariableOp.create(scope, resource, value);
    }
    public Barrier barrier(List<DataType<?>> componentTypes, Barrier.Options... options) {
        return Barrier.create(scope, componentTypes, options);
    }
    public BarrierClose barrierClose(Operand<TString> handle, BarrierClose.Options... options) {
        return BarrierClose.create(scope, handle, options);
    }
    public BarrierIncompleteSize barrierIncompleteSize(Operand<TString> handle) {
        return BarrierIncompleteSize.create(scope, handle);
    }
    public <T extends TType> BarrierInsertMany barrierInsertMany(Operand<TString> handle, Operand<TString> keys, Operand<T> values, Long componentIndex) {
        return BarrierInsertMany.create(scope, handle, keys, values, componentIndex);
    }
    public BarrierReadySize barrierReadySize(Operand<TString> handle) {
        return BarrierReadySize.create(scope, handle);
    }
    public BarrierTakeMany barrierTakeMany(Operand<TString> handle, Operand<TInt32> numElements, List<DataType<?>> componentTypes, BarrierTakeMany.Options... options) {
        return BarrierTakeMany.create(scope, handle, numElements, componentTypes, options);
    }
    public Batch batch(Iterable<Operand<?>> inTensors, Long numBatchThreads, Long maxBatchSize, Long batchTimeoutMicros, Long gradTimeoutMicros, Batch.Options... options) {
        return Batch.create(scope, inTensors, numBatchThreads, maxBatchSize, batchTimeoutMicros, gradTimeoutMicros, options);
    }
    public <T extends TType, U extends TNumber> BatchToSpace<T> batchToSpace(Operand<T> input, Operand<U> crops, Long blockSize) {
        return BatchToSpace.create(scope, input, crops, blockSize);
    }
    public <T extends TType, U extends TNumber, V extends TNumber> BatchToSpaceNd<T> batchToSpaceNd(Operand<T> input, Operand<U> blockShape, Operand<V> crops) {
        return BatchToSpaceNd.create(scope, input, blockShape, crops);
    }
    public <U extends TType, T extends TType> Bitcast<U> bitcast(Operand<T> input, DataType<U> type) {
        return Bitcast.create(scope, input, type);
    }
    public <T extends TNumber> BroadcastDynamicShape<T> broadcastDynamicShape(Operand<T> s0, Operand<T> s1) {
        return BroadcastDynamicShape.create(scope, s0, s1);
    }
    public <T extends TType, U extends TNumber> BroadcastTo<T> broadcastTo(Operand<T> input, Operand<U> shape) {
        return BroadcastTo.create(scope, input, shape);
    }
    public <T extends TNumber> Bucketize bucketize(Operand<T> input, List<Float> boundaries) {
        return Bucketize.create(scope, input, boundaries);
    }
    public <T extends TType> ClipByValue<T> clipByValue(Operand<T> t, Operand<T> clipValueMin, Operand<T> clipValueMax) {
        return ClipByValue.create(scope, t, clipValueMin, clipValueMax);
    }
    public <T extends TType, U extends TNumber> Concat<T> concat(Iterable<Operand<T>> values, Operand<U> axis) {
        return Concat.create(scope, values, axis);
    }
    public Constant<TInt32> constant(int[] data) {
        return Constant.vectorOf(scope, data);
    }
    public Constant<TInt32> constant(int[][][] data) {
        return Constant.tensorOf(scope, data);
    }
    public Constant<TFloat64> constant(double data) {
        return Constant.scalarOf(scope, data);
    }
    public Constant<TInt64> constant(long[][][][][] data) {
        return Constant.tensorOf(scope, data);
    }
    public Constant<TBool> constant(boolean[][][][][] data) {
        return Constant.tensorOf(scope, data);
    }
    public Constant<TInt32> constant(int[][][][] data) {
        return Constant.tensorOf(scope, data);
    }
    public Constant<TFloat32> constant(float[][][][][][] data) {
        return Constant.tensorOf(scope, data);
    }
    public Constant<TString> constant(NdArray<String> data) {
        return Constant.tensorOf(scope, data);
    }
    public Constant<TUint8> constant(byte data) {
        return Constant.scalarOf(scope, data);
    }
    public Constant<TFloat64> constant(DoubleNdArray data) {
        return Constant.tensorOf(scope, data);
    }
    public Constant<TBool> constant(boolean[][][] data) {
        return Constant.tensorOf(scope, data);
    }
    public Constant<TFloat32> constant(float[][][][] data) {
        return Constant.tensorOf(scope, data);
    }
    public Constant<TInt64> constant(long[][] data) {
        return Constant.tensorOf(scope, data);
    }
    public Constant<TInt64> constant(LongNdArray data) {
        return Constant.tensorOf(scope, data);
    }
    public Constant<TUint8> constant(byte[][][][][] data) {
        return Constant.tensorOf(scope, data);
    }
    public Constant<TFloat32> constant(float[][] data) {
        return Constant.tensorOf(scope, data);
    }
    public Constant<TUint8> constant(byte[][] data) {
        return Constant.tensorOf(scope, data);
    }
    public Constant<TFloat64> constant(double[][][][][] data) {
        return Constant.tensorOf(scope, data);
    }
    public Constant<TFloat32> constant(float[][][] data) {
        return Constant.tensorOf(scope, data);
    }
    public Constant<TUint8> constant(byte[] data) {
        return Constant.vectorOf(scope, data);
    }
    public Constant<TFloat32> constant(float[] data) {
        return Constant.vectorOf(scope, data);
    }
    public Constant<TBool> constant(boolean[][] data) {
        return Constant.tensorOf(scope, data);
    }
    public Constant<TUint8> constant(ByteNdArray data) {
        return Constant.tensorOf(scope, data);
    }
    public Constant<TInt32> constant(IntNdArray data) {
        return Constant.tensorOf(scope, data);
    }
    public Constant<TString> constant(String data) {
        return Constant.scalarOf(scope, data);
    }
    public Constant<TFloat64> constant(double[][][][] data) {
        return Constant.tensorOf(scope, data);
    }
    public Constant<TFloat64> constant(double[][] data) {
        return Constant.tensorOf(scope, data);
    }
    public Constant<TInt32> constant(int data) {
        return Constant.scalarOf(scope, data);
    }
    public Constant<TUint8> constant(byte[][][][] data) {
        return Constant.tensorOf(scope, data);
    }
    public Constant<TInt32> constant(int[][][][][][] data) {
        return Constant.tensorOf(scope, data);
    }
    public Constant<TInt64> constant(long data) {
        return Constant.scalarOf(scope, data);
    }
    public Constant<TFloat32> constant(float data) {
        return Constant.scalarOf(scope, data);
    }
    public Constant<TFloat32> constant(float[][][][][] data) {
        return Constant.tensorOf(scope, data);
    }
    public Constant<TFloat64> constant(double[][][] data) {
        return Constant.tensorOf(scope, data);
    }
    public Constant<TInt64> constant(long[][][][][][] data) {
        return Constant.tensorOf(scope, data);
    }
    public Constant<TInt64> constant(long[][][][] data) {
        return Constant.tensorOf(scope, data);
    }
    public Constant<TInt64> constant(long[] data) {
        return Constant.vectorOf(scope, data);
    }
    public Constant<TBool> constant(boolean[] data) {
        return Constant.vectorOf(scope, data);
    }
    public Constant<TUint8> constant(byte[][][] data) {
        return Constant.tensorOf(scope, data);
    }
    public Constant<TUint8> constant(byte[][][][][][] data) {
        return Constant.tensorOf(scope, data);
    }
    public Constant<TInt32> constant(int[][] data) {
        return Constant.tensorOf(scope, data);
    }
    public Constant<TInt32> constant(int[][][][][] data) {
        return Constant.tensorOf(scope, data);
    }
    public Constant<TFloat32> constant(FloatNdArray data) {
        return Constant.tensorOf(scope, data);
    }
    public Constant<TFloat64> constant(double[] data) {
        return Constant.vectorOf(scope, data);
    }
    public Constant<TBool> constant(boolean[][][][][][] data) {
        return Constant.tensorOf(scope, data);
    }
    public Constant<TFloat64> constant(double[][][][][][] data) {
        return Constant.tensorOf(scope, data);
    }
    public Constant<TBool> constant(boolean data) {
        return Constant.scalarOf(scope, data);
    }
    public Constant<TBool> constant(boolean[][][][] data) {
        return Constant.tensorOf(scope, data);
    }
    public Constant<TInt64> constant(long[][][] data) {
        return Constant.tensorOf(scope, data);
    }
    public Constant<TBool> constant(BooleanNdArray data) {
        return Constant.tensorOf(scope, data);
    }
    public Constant<TInt64> constant(Shape shape) {
        return Constant.tensorOf(scope, shape);
    }
    public <T extends TType> Constant<T> constant(Tensor<T> tensor) {
        return Constant.create(scope, tensor);
    }
    public Constant<TString> constant(Charset charset, String[] data) {
        return Constant.vectorOf(scope, charset, data);
    }
    public Constant<TString> constant(Charset charset, String data) {
        return Constant.scalarOf(scope, charset, data);
    }
    public Constant<TString> constant(Charset charset, NdArray<String> data) {
        return Constant.tensorOf(scope, charset, data);
    }
    public Constant<TInt32> constant(Shape shape, IntDataBuffer data) {
        return Constant.tensorOf(scope, shape, data);
    }
    public Constant<TFloat64> constant(Shape shape, DoubleDataBuffer data) {
        return Constant.tensorOf(scope, shape, data);
    }
    public Constant<TBool> constant(Shape shape, BooleanDataBuffer data) {
        return Constant.tensorOf(scope, shape, data);
    }
    public Constant<TFloat32> constant(Shape shape, FloatDataBuffer data) {
        return Constant.tensorOf(scope, shape, data);
    }
    public Constant<TInt64> constant(Shape shape, LongDataBuffer data) {
        return Constant.tensorOf(scope, shape, data);
    }
    public Constant<TUint8> constant(Shape shape, ByteDataBuffer data) {
        return Constant.tensorOf(scope, shape, data);
    }
    public Constant<TString> constant(Shape shape, DataBuffer<String> data) {
        return Constant.tensorOf(scope, shape, data);
    }
    public Constant<TString> constant(Charset charset, Shape shape, DataBuffer<String> data) {
        return Constant.tensorOf(scope, charset, shape, data);
    }
    public <T extends TType> Constant<T> constant(DataType<T> type, Shape shape, ByteDataBuffer data) {
        return Constant.tensorOf(scope, type, shape, data);
    }
    public ConsumeMutexLock consumeMutexLock(Operand<?> mutexLock) {
        return ConsumeMutexLock.create(scope, mutexLock);
    }
    public ControlTrigger controlTrigger() {
        return ControlTrigger.create(scope);
    }
    public <T extends TNumber> CountUpTo<T> countUpTo(Operand<T> ref, Long limit) {
        return CountUpTo.create(scope, ref, limit);
    }
    public <T extends TType> DeepCopy<T> deepCopy(Operand<T> x) {
        return DeepCopy.create(scope, x);
    }
    public DeleteSessionTensor deleteSessionTensor(Operand<TString> handle) {
        return DeleteSessionTensor.create(scope, handle);
    }
    public DestroyResourceOp destroyResourceOp(Operand<?> resource, DestroyResourceOp.Options... options) {
        return DestroyResourceOp.create(scope, resource, options);
    }
    public <T extends TType> DestroyTemporaryVariable<T> destroyTemporaryVariable(Operand<T> ref, String varName) {
        return DestroyTemporaryVariable.create(scope, ref, varName);
    }
    public <T extends TType> DynamicPartition<T> dynamicPartition(Operand<T> data, Operand<TInt32> partitions, Long numPartitions) {
        return DynamicPartition.create(scope, data, partitions, numPartitions);
    }
    public <T extends TType> DynamicStitch<T> dynamicStitch(Iterable<Operand<TInt32>> indices, Iterable<Operand<T>> data) {
        return DynamicStitch.create(scope, indices, data);
    }
    public <T extends TType> EditDistance editDistance(Operand<TInt64> hypothesisIndices, Operand<T> hypothesisValues, Operand<TInt64> hypothesisShape, Operand<TInt64> truthIndices, Operand<T> truthValues, Operand<TInt64> truthShape, EditDistance.Options... options) {
        return EditDistance.create(scope, hypothesisIndices, hypothesisValues, hypothesisShape, truthIndices, truthValues, truthShape, options);
    }
    public <T extends TType> Empty<T> empty(Operand<TInt32> shape, DataType<T> dtype, Empty.Options... options) {
        return Empty.create(scope, shape, dtype, options);
    }
    public <T extends TNumber, U extends TType> EmptyTensorList emptyTensorList(Operand<T> elementShape, Operand<TInt32> maxNumElements, DataType<U> elementDtype) {
        return EmptyTensorList.create(scope, elementShape, maxNumElements, elementDtype);
    }
    public <T extends TType> EnsureShape<T> ensureShape(Operand<T> input, Shape shape) {
        return EnsureShape.create(scope, input, shape);
    }
    public <T extends TType, U extends TNumber> ExpandDims<T> expandDims(Operand<T> input, Operand<U> axis) {
        return ExpandDims.create(scope, input, axis);
    }
    public <T extends TNumber> ExtractVolumePatches<T> extractVolumePatches(Operand<T> input, List<Long> ksizes, List<Long> strides, String padding) {
        return ExtractVolumePatches.create(scope, input, ksizes, strides, padding);
    }
    public <U extends TType, T extends TNumber> Fill<U> fill(Operand<T> dims, Operand<U> value) {
        return Fill.create(scope, dims, value);
    }
    public <T extends TType> Fingerprint fingerprint(Operand<T> data, Operand<TString> method) {
        return Fingerprint.create(scope, data, method);
    }
    public <T extends TType, U extends TNumber, V extends TNumber> Gather<T> gather(Operand<T> params, Operand<U> indices, Operand<V> axis, Gather.Options... options) {
        return Gather.create(scope, params, indices, axis, options);
    }
    public <T extends TType, U extends TNumber> GatherNd<T> gatherNd(Operand<T> params, Operand<U> indices) {
        return GatherNd.create(scope, params, indices);
    }
    public <T extends TType> GetSessionHandle getSessionHandle(Operand<T> value) {
        return GetSessionHandle.create(scope, value);
    }
    public <T extends TType> GetSessionTensor<T> getSessionTensor(Operand<TString> handle, DataType<T> dtype) {
        return GetSessionTensor.create(scope, handle, dtype);
    }
    public Gradients gradients(Iterable<? extends Operand<?>> y, Iterable<? extends Operand<?>> x, Gradients.Options... options) {
        return Gradients.create(scope, y, x, options);
    }
    public Gradients gradients(Operand<?> y, Iterable<? extends Operand<?>> x, Gradients.Options... options) {
        return Gradients.create(scope, y, x, options);
    }
    public <T extends TType> GuaranteeConst<T> guaranteeConst(Operand<T> input) {
        return GuaranteeConst.create(scope, input);
    }
    public <T extends TType, U extends TType> HashTable hashTable(DataType<T> keyDtype, DataType<U> valueDtype, HashTable.Options... options) {
        return HashTable.create(scope, keyDtype, valueDtype, options);
    }
    public <T extends TNumber> HistogramFixedWidth<TInt32> histogramFixedWidth(Operand<T> values, Operand<T> valueRange, Operand<TInt32> nbins) {
        return HistogramFixedWidth.create(scope, values, valueRange, nbins);
    }
    public <U extends TNumber, T extends TNumber> HistogramFixedWidth<U> histogramFixedWidth(Operand<T> values, Operand<T> valueRange, Operand<TInt32> nbins, DataType<U> dtype) {
        return HistogramFixedWidth.create(scope, values, valueRange, nbins, dtype);
    }
    public <T extends TType> Identity<T> identity(Operand<T> input) {
        return Identity.create(scope, input);
    }
    public IdentityN identityN(Iterable<Operand<?>> input) {
        return IdentityN.create(scope, input);
    }
    public <T extends TType> ImmutableConst<T> immutableConst(DataType<T> dtype, Shape shape, String memoryRegionName) {
        return ImmutableConst.create(scope, dtype, shape, memoryRegionName);
    }
    public Init init() {
        return Init.create(scope);
    }
    public void initAdd(Op initializer) {
        Init.add(scope, initializer);
    }
    public <T extends TType, U extends TType> InitializeTable initializeTable(Operand<?> tableHandle, Operand<T> keys, Operand<U> values) {
        return InitializeTable.create(scope, tableHandle, keys, values);
    }
    public InitializeTableFromTextFile initializeTableFromTextFile(Operand<?> tableHandle, Operand<TString> filename, Long keyIndex, Long valueIndex, InitializeTableFromTextFile.Options... options) {
        return InitializeTableFromTextFile.create(scope, tableHandle, filename, keyIndex, valueIndex, options);
    }
    public <T extends TType> InplaceAdd<T> inplaceAdd(Operand<T> x, Operand<TInt32> i, Operand<T> v) {
        return InplaceAdd.create(scope, x, i, v);
    }
    public <T extends TType> InplaceSub<T> inplaceSub(Operand<T> x, Operand<TInt32> i, Operand<T> v) {
        return InplaceSub.create(scope, x, i, v);
    }
    public <T extends TType> InplaceUpdate<T> inplaceUpdate(Operand<T> x, Operand<TInt32> i, Operand<T> v) {
        return InplaceUpdate.create(scope, x, i, v);
    }
    public <T extends TType> IsVariableInitialized isVariableInitialized(Operand<T> ref) {
        return IsVariableInitialized.create(scope, ref);
    }
    public <T extends TNumber, U extends TNumber> LinSpace<T> linSpace(Operand<T> start, Operand<T> stop, Operand<U> num) {
        return LinSpace.create(scope, start, stop, num);
    }
    public <T extends TType, U extends TType> LookupTableExport<T, U> lookupTableExport(Operand<?> tableHandle, DataType<T> Tkeys, DataType<U> Tvalues) {
        return LookupTableExport.create(scope, tableHandle, Tkeys, Tvalues);
    }
    public <U extends TType, T extends TType> LookupTableFind<U> lookupTableFind(Operand<?> tableHandle, Operand<T> keys, Operand<U> defaultValue) {
        return LookupTableFind.create(scope, tableHandle, keys, defaultValue);
    }
    public <T extends TType, U extends TType> LookupTableImport lookupTableImport(Operand<?> tableHandle, Operand<T> keys, Operand<U> values) {
        return LookupTableImport.create(scope, tableHandle, keys, values);
    }
    public <T extends TType, U extends TType> LookupTableInsert lookupTableInsert(Operand<?> tableHandle, Operand<T> keys, Operand<U> values) {
        return LookupTableInsert.create(scope, tableHandle, keys, values);
    }
    public LookupTableSize lookupTableSize(Operand<?> tableHandle) {
        return LookupTableSize.create(scope, tableHandle);
    }
    public LoopCond loopCond(Operand<TBool> input) {
        return LoopCond.create(scope, input);
    }
    public MapClear mapClear(List<DataType<?>> dtypes, MapClear.Options... options) {
        return MapClear.create(scope, dtypes, options);
    }
    public MapIncompleteSize mapIncompleteSize(List<DataType<?>> dtypes, MapIncompleteSize.Options... options) {
        return MapIncompleteSize.create(scope, dtypes, options);
    }
    public MapPeek mapPeek(Operand<TInt64> key, Operand<TInt32> indices, List<DataType<?>> dtypes, MapPeek.Options... options) {
        return MapPeek.create(scope, key, indices, dtypes, options);
    }
    public MapSize mapSize(List<DataType<?>> dtypes, MapSize.Options... options) {
        return MapSize.create(scope, dtypes, options);
    }
    public MapStage mapStage(Operand<TInt64> key, Operand<TInt32> indices, Iterable<Operand<?>> values, List<DataType<?>> dtypes, MapStage.Options... options) {
        return MapStage.create(scope, key, indices, values, dtypes, options);
    }
    public MapUnstage mapUnstage(Operand<TInt64> key, Operand<TInt32> indices, List<DataType<?>> dtypes, MapUnstage.Options... options) {
        return MapUnstage.create(scope, key, indices, dtypes, options);
    }
    public MapUnstageNoKey mapUnstageNoKey(Operand<TInt32> indices, List<DataType<?>> dtypes, MapUnstageNoKey.Options... options) {
        return MapUnstageNoKey.create(scope, indices, dtypes, options);
    }
    public <T extends TType, U extends TNumber> Max<T> max(Operand<T> input, Operand<U> axis, Max.Options... options) {
        return Max.create(scope, input, axis, options);
    }
    public <T extends TType> Merge<T> merge(Iterable<Operand<T>> inputs) {
        return Merge.create(scope, inputs);
    }
    public <T extends TType, U extends TNumber> Min<T> min(Operand<T> input, Operand<U> axis, Min.Options... options) {
        return Min.create(scope, input, axis, options);
    }
    public <T extends TType, U extends TNumber> MirrorPad<T> mirrorPad(Operand<T> input, Operand<U> paddings, String mode) {
        return MirrorPad.create(scope, input, paddings, mode);
    }
    public MlirPassthroughOp mlirPassthroughOp(Iterable<Operand<?>> inputs, String mlirModule, List<DataType<?>> Toutputs) {
        return MlirPassthroughOp.create(scope, inputs, mlirModule, Toutputs);
    }
    public <T extends TType, U extends TType> MutableDenseHashTable mutableDenseHashTable(Operand<T> emptyKey, Operand<T> deletedKey, DataType<U> valueDtype, MutableDenseHashTable.Options... options) {
        return MutableDenseHashTable.create(scope, emptyKey, deletedKey, valueDtype, options);
    }
    public <T extends TType, U extends TType> MutableHashTable mutableHashTable(DataType<T> keyDtype, DataType<U> valueDtype, MutableHashTable.Options... options) {
        return MutableHashTable.create(scope, keyDtype, valueDtype, options);
    }
    public <T extends TType, U extends TType> MutableHashTableOfTensors mutableHashTableOfTensors(DataType<T> keyDtype, DataType<U> valueDtype, MutableHashTableOfTensors.Options... options) {
        return MutableHashTableOfTensors.create(scope, keyDtype, valueDtype, options);
    }
    public Mutex mutex(Mutex.Options... options) {
        return Mutex.create(scope, options);
    }
    public MutexLock mutexLock(Operand<?> mutex) {
        return MutexLock.create(scope, mutex);
    }
    public <T extends TType> NextIteration<T> nextIteration(Operand<T> data) {
        return NextIteration.create(scope, data);
    }
    public NoOp noOp() {
        return NoOp.create(scope);
    }
    public <U extends TType, T extends TNumber> OneHot<U> oneHot(Operand<T> indices, Operand<TInt32> depth, Operand<U> onValue, Operand<U> offValue, OneHot.Options... options) {
        return OneHot.create(scope, indices, depth, onValue, offValue, options);
    }
    public <T extends TType> OnesLike<T> onesLike(Operand<T> x) {
        return OnesLike.create(scope, x);
    }
    public OrderedMapClear orderedMapClear(List<DataType<?>> dtypes, OrderedMapClear.Options... options) {
        return OrderedMapClear.create(scope, dtypes, options);
    }
    public OrderedMapIncompleteSize orderedMapIncompleteSize(List<DataType<?>> dtypes, OrderedMapIncompleteSize.Options... options) {
        return OrderedMapIncompleteSize.create(scope, dtypes, options);
    }
    public OrderedMapPeek orderedMapPeek(Operand<TInt64> key, Operand<TInt32> indices, List<DataType<?>> dtypes, OrderedMapPeek.Options... options) {
        return OrderedMapPeek.create(scope, key, indices, dtypes, options);
    }
    public OrderedMapSize orderedMapSize(List<DataType<?>> dtypes, OrderedMapSize.Options... options) {
        return OrderedMapSize.create(scope, dtypes, options);
    }
    public OrderedMapStage orderedMapStage(Operand<TInt64> key, Operand<TInt32> indices, Iterable<Operand<?>> values, List<DataType<?>> dtypes, OrderedMapStage.Options... options) {
        return OrderedMapStage.create(scope, key, indices, values, dtypes, options);
    }
    public OrderedMapUnstage orderedMapUnstage(Operand<TInt64> key, Operand<TInt32> indices, List<DataType<?>> dtypes, OrderedMapUnstage.Options... options) {
        return OrderedMapUnstage.create(scope, key, indices, dtypes, options);
    }
    public OrderedMapUnstageNoKey orderedMapUnstageNoKey(Operand<TInt32> indices, List<DataType<?>> dtypes, OrderedMapUnstageNoKey.Options... options) {
        return OrderedMapUnstageNoKey.create(scope, indices, dtypes, options);
    }
    public <T extends TType, U extends TNumber> Pad<T> pad(Operand<T> input, Operand<U> paddings, Operand<T> constantValues) {
        return Pad.create(scope, input, paddings, constantValues);
    }
    public <T extends TType> ParallelConcat<T> parallelConcat(Iterable<Operand<T>> values, Shape shape) {
        return ParallelConcat.create(scope, values, shape);
    }
    public <T extends TType> ParallelDynamicStitch<T> parallelDynamicStitch(Iterable<Operand<TInt32>> indices, Iterable<Operand<T>> data) {
        return ParallelDynamicStitch.create(scope, indices, data);
    }
    public <T extends TType> Placeholder<T> placeholder(DataType<T> dtype, Placeholder.Options... options) {
        return Placeholder.create(scope, dtype, options);
    }
    public <T extends TType> PlaceholderWithDefault<T> placeholderWithDefault(Operand<T> input, Shape shape) {
        return PlaceholderWithDefault.create(scope, input, shape);
    }
    public Print print(Operand<TString> input, Print.Options... options) {
        return Print.create(scope, input, options);
    }
    public <T extends TType, U extends TNumber> Prod<T> prod(Operand<T> input, Operand<U> axis, Prod.Options... options) {
        return Prod.create(scope, input, axis, options);
    }
    public <T extends TType, U extends TNumber> QuantizedReshape<T> quantizedReshape(Operand<T> tensor, Operand<U> shape, Operand<TFloat32> inputMin, Operand<TFloat32> inputMax) {
        return QuantizedReshape.create(scope, tensor, shape, inputMin, inputMax);
    }
    public <T extends TNumber> Range<T> range(Operand<T> start, Operand<T> limit, Operand<T> delta) {
        return Range.create(scope, start, limit, delta);
    }
    public <T extends TType> Rank rank(Operand<T> input) {
        return Rank.create(scope, input);
    }
    public <T extends TType> ReadVariableOp<T> readVariableOp(Operand<?> resource, DataType<T> dtype) {
        return ReadVariableOp.create(scope, resource, dtype);
    }
    public <T extends TNumber> ReduceAll reduceAll(Operand<TBool> input, Operand<T> axis, ReduceAll.Options... options) {
        return ReduceAll.create(scope, input, axis, options);
    }
    public <T extends TNumber> ReduceAny reduceAny(Operand<TBool> input, Operand<T> axis, ReduceAny.Options... options) {
        return ReduceAny.create(scope, input, axis, options);
    }
    public <T extends TType, U extends TNumber> ReduceMax<T> reduceMax(Operand<T> input, Operand<U> axis, ReduceMax.Options... options) {
        return ReduceMax.create(scope, input, axis, options);
    }
    public <T extends TType, U extends TNumber> ReduceMin<T> reduceMin(Operand<T> input, Operand<U> axis, ReduceMin.Options... options) {
        return ReduceMin.create(scope, input, axis, options);
    }
    public <T extends TType, U extends TNumber> ReduceProd<T> reduceProd(Operand<T> input, Operand<U> axis, ReduceProd.Options... options) {
        return ReduceProd.create(scope, input, axis, options);
    }
    public <T extends TType, U extends TNumber> ReduceSum<T> reduceSum(Operand<T> input, Operand<U> axis, ReduceSum.Options... options) {
        return ReduceSum.create(scope, input, axis, options);
    }
    public <T extends TType> RefNextIteration<T> refNextIteration(Operand<T> data) {
        return RefNextIteration.create(scope, data);
    }
    public <T extends TType> RefSelect<T> refSelect(Operand<TInt32> index, Iterable<Operand<T>> inputs) {
        return RefSelect.create(scope, index, inputs);
    }
    public <T extends TType> RefSwitch<T> refSwitch(Operand<T> data, Operand<TBool> pred) {
        return RefSwitch.create(scope, data, pred);
    }
    public RemoteFusedGraphExecute remoteFusedGraphExecute(Iterable<Operand<?>> inputs, List<DataType<?>> Toutputs, String serializedRemoteFusedGraphExecuteInfo) {
        return RemoteFusedGraphExecute.create(scope, inputs, Toutputs, serializedRemoteFusedGraphExecuteInfo);
    }
    public <T extends TType, U extends TNumber> Reshape<T> reshape(Operand<T> tensor, Operand<U> shape) {
        return Reshape.create(scope, tensor, shape);
    }
    public <T extends TNumber> ResourceCountUpTo<T> resourceCountUpTo(Operand<?> resource, Long limit, DataType<T> T) {
        return ResourceCountUpTo.create(scope, resource, limit, T);
    }
    public <U extends TType, T extends TNumber> ResourceGather<U> resourceGather(Operand<?> resource, Operand<T> indices, DataType<U> dtype, ResourceGather.Options... options) {
        return ResourceGather.create(scope, resource, indices, dtype, options);
    }
    public <U extends TType, T extends TNumber> ResourceGatherNd<U> resourceGatherNd(Operand<?> resource, Operand<T> indices, DataType<U> dtype) {
        return ResourceGatherNd.create(scope, resource, indices, dtype);
    }
    public <T extends TNumber, U extends TType> ResourceScatterAdd resourceScatterAdd(Operand<?> resource, Operand<T> indices, Operand<U> updates) {
        return ResourceScatterAdd.create(scope, resource, indices, updates);
    }
    public <T extends TNumber, U extends TType> ResourceScatterDiv resourceScatterDiv(Operand<?> resource, Operand<T> indices, Operand<U> updates) {
        return ResourceScatterDiv.create(scope, resource, indices, updates);
    }
    public <T extends TNumber, U extends TType> ResourceScatterMax resourceScatterMax(Operand<?> resource, Operand<T> indices, Operand<U> updates) {
        return ResourceScatterMax.create(scope, resource, indices, updates);
    }
    public <T extends TNumber, U extends TType> ResourceScatterMin resourceScatterMin(Operand<?> resource, Operand<T> indices, Operand<U> updates) {
        return ResourceScatterMin.create(scope, resource, indices, updates);
    }
    public <T extends TNumber, U extends TType> ResourceScatterMul resourceScatterMul(Operand<?> resource, Operand<T> indices, Operand<U> updates) {
        return ResourceScatterMul.create(scope, resource, indices, updates);
    }
    public <T extends TNumber, U extends TType> ResourceScatterNdAdd resourceScatterNdAdd(Operand<?> ref, Operand<T> indices, Operand<U> updates, ResourceScatterNdAdd.Options... options) {
        return ResourceScatterNdAdd.create(scope, ref, indices, updates, options);
    }
    public <T extends TNumber, U extends TType> ResourceScatterNdSub resourceScatterNdSub(Operand<?> ref, Operand<T> indices, Operand<U> updates, ResourceScatterNdSub.Options... options) {
        return ResourceScatterNdSub.create(scope, ref, indices, updates, options);
    }
    public <T extends TNumber, U extends TType> ResourceScatterNdUpdate resourceScatterNdUpdate(Operand<?> ref, Operand<T> indices, Operand<U> updates, ResourceScatterNdUpdate.Options... options) {
        return ResourceScatterNdUpdate.create(scope, ref, indices, updates, options);
    }
    public <T extends TNumber, U extends TType> ResourceScatterSub resourceScatterSub(Operand<?> resource, Operand<T> indices, Operand<U> updates) {
        return ResourceScatterSub.create(scope, resource, indices, updates);
    }
    public <T extends TNumber, U extends TType> ResourceScatterUpdate resourceScatterUpdate(Operand<?> resource, Operand<T> indices, Operand<U> updates) {
        return ResourceScatterUpdate.create(scope, resource, indices, updates);
    }
    public <T extends TNumber, U extends TType> ResourceStridedSliceAssign resourceStridedSliceAssign(Operand<?> ref, Operand<T> begin, Operand<T> end, Operand<T> strides, Operand<U> value, ResourceStridedSliceAssign.Options... options) {
        return ResourceStridedSliceAssign.create(scope, ref, begin, end, strides, value, options);
    }
    public <T extends TType, U extends TNumber> Reverse<T> reverse(Operand<T> tensor, Operand<U> axis) {
        return Reverse.create(scope, tensor, axis);
    }
    public <T extends TType, U extends TNumber> ReverseSequence<T> reverseSequence(Operand<T> input, Operand<U> seqLengths, Long seqDim, ReverseSequence.Options... options) {
        return ReverseSequence.create(scope, input, seqLengths, seqDim, options);
    }
    public <T extends TType, U extends TNumber, V extends TNumber> Roll<T> roll(Operand<T> input, Operand<U> shift, Operand<V> axis) {
        return Roll.create(scope, input, shift, axis);
    }
    public Rpc rpc(Operand<TString> address, Operand<TString> method, Operand<TString> request, Rpc.Options... options) {
        return Rpc.create(scope, address, method, request, options);
    }
    public <T extends TType, U extends TNumber> ScatterAdd<T> scatterAdd(Operand<T> ref, Operand<U> indices, Operand<T> updates, ScatterAdd.Options... options) {
        return ScatterAdd.create(scope, ref, indices, updates, options);
    }
    public <T extends TType, U extends TNumber> ScatterDiv<T> scatterDiv(Operand<T> ref, Operand<U> indices, Operand<T> updates, ScatterDiv.Options... options) {
        return ScatterDiv.create(scope, ref, indices, updates, options);
    }
    public <T extends TNumber, U extends TNumber> ScatterMax<T> scatterMax(Operand<T> ref, Operand<U> indices, Operand<T> updates, ScatterMax.Options... options) {
        return ScatterMax.create(scope, ref, indices, updates, options);
    }
    public <T extends TNumber, U extends TNumber> ScatterMin<T> scatterMin(Operand<T> ref, Operand<U> indices, Operand<T> updates, ScatterMin.Options... options) {
        return ScatterMin.create(scope, ref, indices, updates, options);
    }
    public <T extends TType, U extends TNumber> ScatterMul<T> scatterMul(Operand<T> ref, Operand<U> indices, Operand<T> updates, ScatterMul.Options... options) {
        return ScatterMul.create(scope, ref, indices, updates, options);
    }
    public <U extends TType, T extends TNumber> ScatterNd<U> scatterNd(Operand<T> indices, Operand<U> updates, Operand<T> shape) {
        return ScatterNd.create(scope, indices, updates, shape);
    }
    public <T extends TType, U extends TNumber> ScatterNdAdd<T> scatterNdAdd(Operand<T> ref, Operand<U> indices, Operand<T> updates, ScatterNdAdd.Options... options) {
        return ScatterNdAdd.create(scope, ref, indices, updates, options);
    }
    public <T extends TType, U extends TNumber> ScatterNdNonAliasingAdd<T> scatterNdNonAliasingAdd(Operand<T> input, Operand<U> indices, Operand<T> updates) {
        return ScatterNdNonAliasingAdd.create(scope, input, indices, updates);
    }
    public <T extends TType, U extends TNumber> ScatterNdSub<T> scatterNdSub(Operand<T> ref, Operand<U> indices, Operand<T> updates, ScatterNdSub.Options... options) {
        return ScatterNdSub.create(scope, ref, indices, updates, options);
    }
    public <T extends TType, U extends TNumber> ScatterNdUpdate<T> scatterNdUpdate(Operand<T> ref, Operand<U> indices, Operand<T> updates, ScatterNdUpdate.Options... options) {
        return ScatterNdUpdate.create(scope, ref, indices, updates, options);
    }
    public <T extends TType, U extends TNumber> ScatterSub<T> scatterSub(Operand<T> ref, Operand<U> indices, Operand<T> updates, ScatterSub.Options... options) {
        return ScatterSub.create(scope, ref, indices, updates, options);
    }
    public <T extends TType, U extends TNumber> ScatterUpdate<T> scatterUpdate(Operand<T> ref, Operand<U> indices, Operand<T> updates, ScatterUpdate.Options... options) {
        return ScatterUpdate.create(scope, ref, indices, updates, options);
    }
    public <T extends TType> Select<T> select(Operand<TBool> condition, Operand<T> t, Operand<T> e) {
        return Select.create(scope, condition, t, e);
    }
    public <T extends TType> SetDiff1d<T, TInt32> setDiff1d(Operand<T> x, Operand<T> y) {
        return SetDiff1d.create(scope, x, y);
    }
    public <T extends TType, U extends TNumber> SetDiff1d<T, U> setDiff1d(Operand<T> x, Operand<T> y, DataType<U> outIdx) {
        return SetDiff1d.create(scope, x, y, outIdx);
    }
    public <T extends TType> SetSize setSize(Operand<TInt64> setIndices, Operand<T> setValues, Operand<TInt64> setShape, SetSize.Options... options) {
        return SetSize.create(scope, setIndices, setValues, setShape, options);
    }
    public <T extends TType> org.tensorflow.op.core.Shape<TInt32> shape(Operand<T> input) {
        return org.tensorflow.op.core.Shape.create(scope, input);
    }
    public <U extends TNumber, T extends TType> org.tensorflow.op.core.Shape<U> shape(Operand<T> input, DataType<U> outType) {
        return org.tensorflow.op.core.Shape.create(scope, input, outType);
    }
    public <T extends TType> ShapeN<TInt32> shapeN(Iterable<Operand<T>> input) {
        return ShapeN.create(scope, input);
    }
    public <U extends TNumber, T extends TType> ShapeN<U> shapeN(Iterable<Operand<T>> input, DataType<U> outType) {
        return ShapeN.create(scope, input, outType);
    }
    public <T extends TType> Size<TInt32> size(Operand<T> input) {
        return Size.create(scope, input);
    }
    public <U extends TNumber, T extends TType> Size<U> size(Operand<T> input, DataType<U> outType) {
        return Size.create(scope, input, outType);
    }
    public Skipgram skipgram(String filename, Long batchSize, Skipgram.Options... options) {
        return Skipgram.create(scope, filename, batchSize, options);
    }
    public <T extends TType, U extends TNumber> Slice<T> slice(Operand<T> input, Operand<U> begin, Operand<U> size) {
        return Slice.create(scope, input, begin, size);
    }
    public <T extends TType> Snapshot<T> snapshot(Operand<T> input) {
        return Snapshot.create(scope, input);
    }
    public <T extends TType, U extends TNumber, V extends TNumber> SpaceToBatchNd<T> spaceToBatchNd(Operand<T> input, Operand<U> blockShape, Operand<V> paddings) {
        return SpaceToBatchNd.create(scope, input, blockShape, paddings);
    }
    public <T extends TType> Split<T> split(Operand<TInt32> axis, Operand<T> value, Long numSplit) {
        return Split.create(scope, axis, value, numSplit);
    }
    public <T extends TType, U extends TNumber> SplitV<T> splitV(Operand<T> value, Operand<U> sizeSplits, Operand<TInt32> axis, Long numSplit) {
        return SplitV.create(scope, value, sizeSplits, axis, numSplit);
    }
    public <T extends TType> Squeeze<T> squeeze(Operand<T> input, Squeeze.Options... options) {
        return Squeeze.create(scope, input, options);
    }
    public <T extends TType> Stack<T> stack(Iterable<Operand<T>> values, Stack.Options... options) {
        return Stack.create(scope, values, options);
    }
    public Stage stage(Iterable<Operand<?>> values, Stage.Options... options) {
        return Stage.create(scope, values, options);
    }
    public StageClear stageClear(List<DataType<?>> dtypes, StageClear.Options... options) {
        return StageClear.create(scope, dtypes, options);
    }
    public StagePeek stagePeek(Operand<TInt32> index, List<DataType<?>> dtypes, StagePeek.Options... options) {
        return StagePeek.create(scope, index, dtypes, options);
    }
    public StageSize stageSize(List<DataType<?>> dtypes, StageSize.Options... options) {
        return StageSize.create(scope, dtypes, options);
    }
    public <T extends TType> StopGradient<T> stopGradient(Operand<T> input) {
        return StopGradient.create(scope, input);
    }
    public <T extends TType, U extends TNumber> StridedSlice<T> stridedSlice(Operand<T> input, Operand<U> begin, Operand<U> end, Operand<U> strides, StridedSlice.Options... options) {
        return StridedSlice.create(scope, input, begin, end, strides, options);
    }
    public <T extends TType, U extends TNumber> StridedSliceAssign<T> stridedSliceAssign(Operand<T> ref, Operand<U> begin, Operand<U> end, Operand<U> strides, Operand<T> value, StridedSliceAssign.Options... options) {
        return StridedSliceAssign.create(scope, ref, begin, end, strides, value, options);
    }
    public <U extends TType, T extends TNumber> StridedSliceGrad<U> stridedSliceGrad(Operand<T> shape, Operand<T> begin, Operand<T> end, Operand<T> strides, Operand<U> dy, StridedSliceGrad.Options... options) {
        return StridedSliceGrad.create(scope, shape, begin, end, strides, dy, options);
    }
    public <T extends TType, U extends TNumber> Sum<T> sum(Operand<T> input, Operand<U> axis, Sum.Options... options) {
        return Sum.create(scope, input, axis, options);
    }
    public <T extends TType> SwitchCond<T> switchCond(Operand<T> data, Operand<TBool> pred) {
        return SwitchCond.create(scope, data, pred);
    }
    public <T extends TType> TemporaryVariable<T> temporaryVariable(Shape shape, DataType<T> dtype, TemporaryVariable.Options... options) {
        return TemporaryVariable.create(scope, shape, dtype, options);
    }
    public <T extends TType> TensorArray tensorArray(Operand<TInt32> size, DataType<T> dtype, TensorArray.Options... options) {
        return TensorArray.create(scope, size, dtype, options);
    }
    public TensorArrayClose tensorArrayClose(Operand<?> handle) {
        return TensorArrayClose.create(scope, handle);
    }
    public <T extends TType> TensorArrayConcat<T> tensorArrayConcat(Operand<?> handle, Operand<TFloat32> flowIn, DataType<T> dtype, TensorArrayConcat.Options... options) {
        return TensorArrayConcat.create(scope, handle, flowIn, dtype, options);
    }
    public <T extends TType> TensorArrayGather<T> tensorArrayGather(Operand<?> handle, Operand<TInt32> indices, Operand<TFloat32> flowIn, DataType<T> dtype, TensorArrayGather.Options... options) {
        return TensorArrayGather.create(scope, handle, indices, flowIn, dtype, options);
    }
    public TensorArrayGrad tensorArrayGrad(Operand<?> handle, Operand<TFloat32> flowIn, String source) {
        return TensorArrayGrad.create(scope, handle, flowIn, source);
    }
    public TensorArrayGradWithShape tensorArrayGradWithShape(Operand<?> handle, Operand<TFloat32> flowIn, Operand<TInt32> shapeToPrepend, String source) {
        return TensorArrayGradWithShape.create(scope, handle, flowIn, shapeToPrepend, source);
    }
    public <T extends TType> TensorArrayPack<T> tensorArrayPack(Operand<TString> handle, Operand<TFloat32> flowIn, DataType<T> dtype, TensorArrayPack.Options... options) {
        return TensorArrayPack.create(scope, handle, flowIn, dtype, options);
    }
    public <T extends TType> TensorArrayRead<T> tensorArrayRead(Operand<?> handle, Operand<TInt32> index, Operand<TFloat32> flowIn, DataType<T> dtype) {
        return TensorArrayRead.create(scope, handle, index, flowIn, dtype);
    }
    public <T extends TType> TensorArrayScatter tensorArrayScatter(Operand<?> handle, Operand<TInt32> indices, Operand<T> value, Operand<TFloat32> flowIn) {
        return TensorArrayScatter.create(scope, handle, indices, value, flowIn);
    }
    public TensorArraySize tensorArraySize(Operand<?> handle, Operand<TFloat32> flowIn) {
        return TensorArraySize.create(scope, handle, flowIn);
    }
    public <T extends TType> TensorArraySplit tensorArraySplit(Operand<?> handle, Operand<T> value, Operand<TInt64> lengths, Operand<TFloat32> flowIn) {
        return TensorArraySplit.create(scope, handle, value, lengths, flowIn);
    }
    public <T extends TType> TensorArrayUnpack tensorArrayUnpack(Operand<TString> handle, Operand<T> value, Operand<TFloat32> flowIn) {
        return TensorArrayUnpack.create(scope, handle, value, flowIn);
    }
    public <T extends TType> TensorArrayWrite tensorArrayWrite(Operand<?> handle, Operand<TInt32> index, Operand<T> value, Operand<TFloat32> flowIn) {
        return TensorArrayWrite.create(scope, handle, index, value, flowIn);
    }
    public <U extends TType, T extends TNumber> TensorListConcat<U> tensorListConcat(Operand<?> inputHandle, Operand<T> elementShape, Operand<TInt64> leadingDims, DataType<U> elementDtype) {
        return TensorListConcat.create(scope, inputHandle, elementShape, leadingDims, elementDtype);
    }
    public <T extends TType> TensorListConcatLists tensorListConcatLists(Operand<?> inputA, Operand<?> inputB, DataType<T> elementDtype) {
        return TensorListConcatLists.create(scope, inputA, inputB, elementDtype);
    }
    public <T extends TNumber> TensorListElementShape<T> tensorListElementShape(Operand<?> inputHandle, DataType<T> shapeType) {
        return TensorListElementShape.create(scope, inputHandle, shapeType);
    }
    public <T extends TType, U extends TNumber> TensorListFromTensor tensorListFromTensor(Operand<T> tensor, Operand<U> elementShape) {
        return TensorListFromTensor.create(scope, tensor, elementShape);
    }
    public <T extends TType> TensorListGather<T> tensorListGather(Operand<?> inputHandle, Operand<TInt32> indices, Operand<TInt32> elementShape, DataType<T> elementDtype) {
        return TensorListGather.create(scope, inputHandle, indices, elementShape, elementDtype);
    }
    public <T extends TType> TensorListGetItem<T> tensorListGetItem(Operand<?> inputHandle, Operand<TInt32> index, Operand<TInt32> elementShape, DataType<T> elementDtype) {
        return TensorListGetItem.create(scope, inputHandle, index, elementShape, elementDtype);
    }
    public TensorListLength tensorListLength(Operand<?> inputHandle) {
        return TensorListLength.create(scope, inputHandle);
    }
    public <T extends TType> TensorListPopBack<T> tensorListPopBack(Operand<?> inputHandle, Operand<TInt32> elementShape, DataType<T> elementDtype) {
        return TensorListPopBack.create(scope, inputHandle, elementShape, elementDtype);
    }
    public <T extends TType> TensorListPushBack tensorListPushBack(Operand<?> inputHandle, Operand<T> tensor) {
        return TensorListPushBack.create(scope, inputHandle, tensor);
    }
    public <T extends TType> TensorListPushBackBatch tensorListPushBackBatch(Operand<?> inputHandles, Operand<T> tensor) {
        return TensorListPushBackBatch.create(scope, inputHandles, tensor);
    }
    public <T extends TNumber, U extends TType> TensorListReserve tensorListReserve(Operand<T> elementShape, Operand<TInt32> numElements, DataType<U> elementDtype) {
        return TensorListReserve.create(scope, elementShape, numElements, elementDtype);
    }
    public TensorListResize tensorListResize(Operand<?> inputHandle, Operand<TInt32> size) {
        return TensorListResize.create(scope, inputHandle, size);
    }
    public <T extends TType, U extends TNumber> TensorListScatter tensorListScatter(Operand<T> tensor, Operand<TInt32> indices, Operand<U> elementShape, Operand<TInt32> numElements) {
        return TensorListScatter.create(scope, tensor, indices, elementShape, numElements);
    }
    public <T extends TType> TensorListScatterIntoExistingList tensorListScatterIntoExistingList(Operand<?> inputHandle, Operand<T> tensor, Operand<TInt32> indices) {
        return TensorListScatterIntoExistingList.create(scope, inputHandle, tensor, indices);
    }
    public <T extends TType> TensorListSetItem tensorListSetItem(Operand<?> inputHandle, Operand<TInt32> index, Operand<T> item) {
        return TensorListSetItem.create(scope, inputHandle, index, item);
    }
    public <T extends TType, U extends TNumber> TensorListSplit tensorListSplit(Operand<T> tensor, Operand<U> elementShape, Operand<TInt64> lengths) {
        return TensorListSplit.create(scope, tensor, elementShape, lengths);
    }
    public <T extends TType> TensorListStack<T> tensorListStack(Operand<?> inputHandle, Operand<TInt32> elementShape, DataType<T> elementDtype, TensorListStack.Options... options) {
        return TensorListStack.create(scope, inputHandle, elementShape, elementDtype, options);
    }
    public <T extends TType, U extends TNumber> TensorScatterNdAdd<T> tensorScatterNdAdd(Operand<T> tensor, Operand<U> indices, Operand<T> updates) {
        return TensorScatterNdAdd.create(scope, tensor, indices, updates);
    }
    public <T extends TType, U extends TNumber> TensorScatterNdSub<T> tensorScatterNdSub(Operand<T> tensor, Operand<U> indices, Operand<T> updates) {
        return TensorScatterNdSub.create(scope, tensor, indices, updates);
    }
    public <T extends TType, U extends TNumber> TensorScatterNdUpdate<T> tensorScatterNdUpdate(Operand<T> tensor, Operand<U> indices, Operand<T> updates) {
        return TensorScatterNdUpdate.create(scope, tensor, indices, updates);
    }
    public <T extends TType, U extends TNumber> TensorStridedSliceUpdate<T> tensorStridedSliceUpdate(Operand<T> input, Operand<U> begin, Operand<U> end, Operand<U> strides, Operand<T> value, TensorStridedSliceUpdate.Options... options) {
        return TensorStridedSliceUpdate.create(scope, input, begin, end, strides, value, options);
    }
    public <T extends TType, U extends TNumber> Tile<T> tile(Operand<T> input, Operand<U> multiples) {
        return Tile.create(scope, input, multiples);
    }
    public Timestamp timestamp() {
        return Timestamp.create(scope);
    }
    public TryRpc tryRpc(Operand<TString> address, Operand<TString> method, Operand<TString> request, TryRpc.Options... options) {
        return TryRpc.create(scope, address, method, request, options);
    }
    public <T extends TType> Unbatch<T> unbatch(Operand<T> batchedTensor, Operand<TInt64> batchIndex, Operand<TInt64> id, Long timeoutMicros, Unbatch.Options... options) {
        return Unbatch.create(scope, batchedTensor, batchIndex, id, timeoutMicros, options);
    }
    public <T extends TType> UnbatchGrad<T> unbatchGrad(Operand<T> originalInput, Operand<TInt64> batchIndex, Operand<T> grad, Operand<TInt64> id, UnbatchGrad.Options... options) {
        return UnbatchGrad.create(scope, originalInput, batchIndex, grad, id, options);
    }
    public <T extends TType, U extends TNumber> Unique<T, TInt32> unique(Operand<T> x, Operand<U> axis) {
        return Unique.create(scope, x, axis);
    }
    public <T extends TType, V extends TNumber, U extends TNumber> Unique<T, V> unique(Operand<T> x, Operand<U> axis, DataType<V> outIdx) {
        return Unique.create(scope, x, axis, outIdx);
    }
    public <T extends TType, U extends TNumber> UniqueWithCounts<T, TInt32> uniqueWithCounts(Operand<T> x, Operand<U> axis) {
        return UniqueWithCounts.create(scope, x, axis);
    }
    public <T extends TType, V extends TNumber, U extends TNumber> UniqueWithCounts<T, V> uniqueWithCounts(Operand<T> x, Operand<U> axis, DataType<V> outIdx) {
        return UniqueWithCounts.create(scope, x, axis, outIdx);
    }
    public <T extends TNumber> UnravelIndex<T> unravelIndex(Operand<T> indices, Operand<T> dims) {
        return UnravelIndex.create(scope, indices, dims);
    }
    public <T extends TType> Unstack<T> unstack(Operand<T> value, Long num, Unstack.Options... options) {
        return Unstack.create(scope, value, num, options);
    }
    public Unstage unstage(List<DataType<?>> dtypes, Unstage.Options... options) {
        return Unstage.create(scope, dtypes, options);
    }
    public <T extends TType> VarHandleOp varHandleOp(DataType<T> dtype, Shape shape, VarHandleOp.Options... options) {
        return VarHandleOp.create(scope, dtype, shape, options);
    }
    public VarIsInitializedOp varIsInitializedOp(Operand<?> resource) {
        return VarIsInitializedOp.create(scope, resource);
    }
    public <T extends TType> Variable<T> variable(Operand<T> init, Variable.Options... options) {
        return Helpers.createVariableWithInit(scope, init, options);
    }
    public <T extends TType> Variable<T> variable(Shape shape, DataType<T> dtype, Variable.Options... options) {
        return Variable.create(scope, shape, dtype, options);
    }
    public VariableShape<TInt32> variableShape(Operand<?> input) {
        return VariableShape.create(scope, input);
    }
    public <T extends TNumber> VariableShape<T> variableShape(Operand<?> input, DataType<T> outType) {
        return VariableShape.create(scope, input, outType);
    }
    public <T extends TType> Where where(Operand<T> condition) {
        return Where.create(scope, condition);
    }
    public <T extends TType, U extends TNumber> Zeros<T> zeros(Operand<U> dims, DataType<T> type) {
        return Zeros.create(scope, dims, type);
    }
    public <T extends TType> ZerosLike<T> zerosLike(Operand<T> x) {
        return ZerosLike.create(scope, x);
    }
    public Ops withSubScope(String childScopeName) {
        return new Ops(scope.withSubScope(childScopeName));
    }
    public Ops withName(String opName) {
        return new Ops(scope.withName(opName));
    }
    public Ops withControlDependencies(Iterable<Op> controls) {
        return new Ops(scope.withControlDependencies(controls));
    }
    public final Scope scope() {
        return scope;
    }
    public static Ops create(ExecutionEnvironment env) {
        return new Ops(new Scope(env));
    }
    public static Ops create() {
        return new Ops(new Scope(EagerSession.getDefault()));
    }
}
